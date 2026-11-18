//! Home's card search bar: typing looks cards up by name, signed in or out, and picking one
//! opens its card page.
//!
//! Typing waits [`timing::CARD_SEARCH_DEBOUNCE`] for the user to rest, then sends one lookup.
//! Every keystroke takes a new ticket, so only the last keystroke's wait sends and only the
//! newest lookup's answer shows. An answer lists up to [`LOOKUP_LIMIT`] cards. A rate-limited
//! lookup waits out the server's `Retry-After` ([`timing::CARD_SEARCH_RATE_LIMIT_WAIT`] when it
//! can't be read), then looks up whatever the box holds by then.
//!
//! The model holds the box and what came back; the client sends the lookup (with the session's
//! token when one is held, so the user's Universes Beyond setting applies), runs the timers and
//! draws the results.

#[cfg(test)]
mod tests;

use std::time::Duration;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::models::lookup::{LOOKUP_LIMIT, LookupName};
use zwipe_core::domain::card::slug::{self, CardSlug};
use zwipe_core::http::contracts::card::HttpCardLookup;

use crate::generation::{Generation, Ticket};
use crate::timing;

/// What the bar shows under the box.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Nothing typed.
    #[default]
    Idle,
    /// A lookup is out.
    Loading,
    /// The answer's cards, in [`State::results`].
    Results,
    /// No card matches.
    NoMatch,
    /// Too many lookups; the bar looks up again once the server's wait is over.
    RateLimited,
    /// The lookup failed; [`Action::Retry`] asks again.
    Failed,
}

/// Why a lookup failed, as the client reads its error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Failure {
    /// 429: too many lookups from this address.
    RateLimited {
        /// The server's `Retry-After`, or `None` when the client can't read it.
        retry_after: Option<Duration>,
    },
    /// Anything else: the network, a server error, a reply that didn't decode.
    Other,
}

/// Where picking a result goes. The client maps each to its own route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// The card page for a name's slug, `/card/<slug>`.
    Card(CardSlug),
    /// One printing's page, for a name with no letter or digit to make a slug from.
    Printing {
        /// The path, [`slug::printing_path`]'s.
        path: String,
    },
}

/// Every tap, key and reply the bar takes.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// The box now holds this text.
    Type(String),
    /// A typing wait is over; the [`Effect::Debounce`] ticket.
    Debounced(Ticket),
    /// A lookup answered.
    Loaded {
        /// The [`Effect::Lookup`] ticket.
        ticket: Ticket,
        /// The cards it found.
        cards: Vec<Card>,
    },
    /// A lookup failed.
    Failed {
        /// The [`Effect::Lookup`] ticket.
        ticket: Ticket,
        /// Why.
        failure: Failure,
    },
    /// A rate limit's wait is over; the [`Effect::Cooldown`] ticket.
    CooledDown(Ticket),
    /// Retry after a failure.
    Retry,
    /// The box's clear button.
    Clear,
    /// A result picked, by its printing id.
    Pick(Uuid),
}

/// What an action asks the client to do.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Waits `wait`, then answers with [`Action::Debounced`] carrying `ticket`.
    Debounce {
        /// The ticket to hand back.
        ticket: Ticket,
        /// How long: [`timing::CARD_SEARCH_DEBOUNCE`].
        wait: Duration,
    },
    /// Sends the lookup, with the session's token when one is held; answers with
    /// [`Action::Loaded`] or [`Action::Failed`] carrying `ticket`.
    Lookup {
        /// The ticket to hand back.
        ticket: Ticket,
        /// What to look up.
        lookup: HttpCardLookup,
    },
    /// Waits `wait`, then answers with [`Action::CooledDown`] carrying `ticket`.
    Cooldown {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The server's `Retry-After`, or [`timing::CARD_SEARCH_RATE_LIMIT_WAIT`].
        wait: Duration,
    },
    /// Opens a card.
    Navigate(Destination),
}

/// The bar for one visit to Home.
#[derive(Debug, Clone, Default)]
pub struct State {
    query: String,
    status: Status,
    results: Vec<Card>,
    tickets: Generation,
    /// The rate limit's wait while it runs: typing waits it out instead of sending.
    cooling: Option<Ticket>,
}

impl State {
    /// An empty bar.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The box's text.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// What the bar shows.
    #[must_use]
    pub fn status(&self) -> Status {
        self.status
    }

    /// The last answer's cards, at most [`LOOKUP_LIMIT`], while [`Status::Results`] shows.
    #[must_use]
    pub fn results(&self) -> &[Card] {
        if self.status == Status::Results {
            &self.results
        } else {
            &[]
        }
    }

    /// Takes one action and answers with what the client should do.
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Type(text) => self.type_text(text),
            Action::Debounced(ticket) => {
                if self.cooling.is_some() || !self.tickets.is_current(ticket) {
                    return Vec::new();
                }
                self.lookup(ticket)
            }
            Action::Loaded { ticket, cards } => {
                if self.tickets.is_current(ticket) && self.status == Status::Loading {
                    self.results = cards.into_iter().take(limit()).collect();
                    self.status = if self.results.is_empty() {
                        Status::NoMatch
                    } else {
                        Status::Results
                    };
                }
                Vec::new()
            }
            Action::Failed { ticket, failure } => self.failed(ticket, failure),
            Action::CooledDown(ticket) => {
                if self.cooling != Some(ticket) {
                    return Vec::new();
                }
                self.cooling = None;
                if self.query.trim().is_empty() {
                    return Vec::new();
                }
                let ticket = self.tickets.next_ticket();
                self.lookup(ticket)
            }
            Action::Retry => {
                if self.status != Status::Failed {
                    return Vec::new();
                }
                let ticket = self.tickets.next_ticket();
                self.lookup(ticket)
            }
            Action::Clear => {
                self.query.clear();
                self.go_idle();
                Vec::new()
            }
            Action::Pick(id) => self
                .results()
                .iter()
                .find(|card| card.scryfall_data.id == id)
                .map(|card| Effect::Navigate(destination(card)))
                .into_iter()
                .collect(),
        }
    }

    fn type_text(&mut self, text: String) -> Vec<Effect> {
        self.query = text;
        if self.query.trim().is_empty() {
            self.go_idle();
            return Vec::new();
        }
        let ticket = self.tickets.next_ticket();
        if self.cooling.is_some() {
            return Vec::new();
        }
        if name_lookup(&self.query).is_none() {
            self.results.clear();
            self.status = Status::NoMatch;
            return Vec::new();
        }
        vec![Effect::Debounce {
            ticket,
            wait: timing::CARD_SEARCH_DEBOUNCE,
        }]
    }

    /// Sends the lookup for the box as it stands under `ticket`.
    fn lookup(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some(lookup) = name_lookup(&self.query) else {
            if self.query.trim().is_empty() {
                self.go_idle();
            } else {
                self.results.clear();
                self.status = Status::NoMatch;
            }
            return Vec::new();
        };
        self.status = Status::Loading;
        vec![Effect::Lookup { ticket, lookup }]
    }

    fn failed(&mut self, ticket: Ticket, failure: Failure) -> Vec<Effect> {
        if !self.tickets.is_current(ticket) || self.status != Status::Loading {
            return Vec::new();
        }
        self.results.clear();
        match failure {
            Failure::RateLimited { retry_after } => {
                self.status = Status::RateLimited;
                let cool = self.tickets.next_ticket();
                self.cooling = Some(cool);
                vec![Effect::Cooldown {
                    ticket: cool,
                    wait: retry_after.unwrap_or(timing::CARD_SEARCH_RATE_LIMIT_WAIT),
                }]
            }
            Failure::Other => {
                self.status = Status::Failed;
                Vec::new()
            }
        }
    }

    /// Nothing to show, and nothing out counts: a newer ticket drops every answer in flight.
    /// A rate limit's wait keeps running, since the server still counts it.
    fn go_idle(&mut self) {
        let _ = self.tickets.next_ticket();
        self.results.clear();
        self.status = Status::Idle;
    }
}

/// [`LOOKUP_LIMIT`] as a count of cards.
fn limit() -> usize {
    usize::try_from(LOOKUP_LIMIT).unwrap_or(usize::MAX)
}

/// The lookup for `text` as a name, or `None` when the server would refuse it (empty once
/// trimmed, too long, holding a control character, or with no letter or digit).
fn name_lookup(text: &str) -> Option<HttpCardLookup> {
    let name = LookupName::try_from(text.to_owned()).ok()?;
    HttpCardLookup::try_new(name, Vec::new(), Vec::new(), Vec::new()).ok()
}

/// Where a picked card opens: its name's card page, or its printing's page for a name with no
/// slug.
fn destination(card: &Card) -> Destination {
    let name = &card.scryfall_data.name;
    match slug::card_slug(name) {
        Some(slug) => Destination::Card(slug),
        None => Destination::Printing {
            path: slug::printing_path(name, card.scryfall_data.id),
        },
    }
}
