//! The card search bar, in the wide nav and in Home's Card search panel: typing looks cards up
//! by name, signed in or out, and picking one opens its card page.
//!
//! Enter opens the one match's card page. With several matches it opens the results page for
//! the box, unless the arrow keys have lit one, which it opens instead; before the answer comes
//! it opens the results page too, which looks the text up itself. Opening anything empties the
//! box.
//!
//! Typing waits [`timing::CARD_SEARCH_DEBOUNCE`] for the user to rest, then sends one lookup.
//! Every keystroke takes a new ticket, so only the last keystroke's wait sends and only the
//! newest lookup's answer shows. An answer shows its first [`CHIP_LIMIT`] cards as chips, and
//! Enter opens the results page with every card it matches. A rate-limited
//! lookup waits out the server's `Retry-After` ([`timing::CARD_SEARCH_RATE_LIMIT_WAIT`] when it
//! can't be read), then looks up whatever the box holds by then.
//!
//! The model holds the box and what came back; the client sends the lookup (with the session's
//! token when one is held, so the user's Universes Beyond setting applies), runs the timers and
//! draws the results.

#[cfg(test)]
mod tests;

use std::fmt::Write as _;
use std::time::Duration;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::models::lookup::{LOOKUP_LIMIT, LookupName};
use zwipe_core::domain::card::slug::{self, CardSlug};
use zwipe_core::http::contracts::card::HttpCardLookup;

use crate::commander::maybeboard;
pub use crate::commander::maybeboard::Step;
use crate::generation::{Generation, Ticket};
use crate::timing;

/// The bar's label.
pub const LABEL: &str = "Search";
/// The box's and its chip's name for assistive tech.
pub const NAME: &str = "Card search";
/// The empty box's hint.
pub const PLACEHOLDER: &str = "Card search";
/// The line while a lookup is out.
pub const SEARCHING: &str = "Searching…";
/// The line when no card matches.
pub const NO_MATCH: &str = "No cards match";
/// The line while a rate limit's wait runs.
pub const RATE_LIMITED: &str = "Too many searches, retrying in a moment";
/// The line when a lookup failed, beside Retry.
pub const FAILED: &str = "Couldn't search";

/// How many of an answer's cards show as chips under the box, in the answer's order.
pub const CHIP_LIMIT: usize = 10;

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
    /// The results page for the box's text, `/search?q=<query>`, trimmed.
    Results {
        /// What was typed.
        query: String,
    },
}

impl Destination {
    /// The destination's path on the web app, for a host that links there rather than routing:
    /// `/card/<slug>`, the printing's path, or `/search?q=<query>` with the query
    /// percent-encoded.
    #[must_use]
    pub fn path(&self) -> String {
        match self {
            Self::Card(slug) => format!("/card/{slug}"),
            Self::Printing { path } => path.clone(),
            Self::Results { query } => format!("/search?q={}", percent_encode(query)),
        }
    }
}

/// `text` for a URL's query value: every byte outside the unreserved set (letters, digits and
/// `-._~`) written as `%XX`, a space as `%20`.
fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            let _ = write!(out, "%{byte:02X}");
        }
    }
    out
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
    /// An arrow key: moves the highlight one result, stopping at either end.
    Step(Step),
    /// Enter in the box: opens the highlighted result, else the results page for the box's
    /// text. A box with nothing to look up, or whose answer matched nothing, opens nothing.
    Enter,
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
    /// The highlighted result's place: the one match when an answer has exactly one, else
    /// none until an arrow key lights one.
    highlight: Option<usize>,
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

    /// The last answer's first [`CHIP_LIMIT`] cards, while [`Status::Results`] shows.
    #[must_use]
    pub fn results(&self) -> &[Card] {
        if self.status == Status::Results {
            &self.results
        } else {
            &[]
        }
    }

    /// The highlighted result's place among [`State::results`], which Enter opens: the match
    /// when an answer has only one, else the one the arrow keys light, and none while no
    /// results show.
    #[must_use]
    pub fn highlight(&self) -> Option<usize> {
        self.highlight.filter(|_| self.status == Status::Results)
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
                    self.results = cards.into_iter().take(CHIP_LIMIT).collect();
                    self.highlight = (self.results.len() == 1).then_some(0);
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
            Action::Pick(id) => {
                let to = self
                    .results()
                    .iter()
                    .find(|card| card.scryfall_data.id == id)
                    .map(destination);
                self.open(to)
            }
            Action::Step(step) => {
                self.highlight =
                    maybeboard::step_highlight(self.highlight(), self.results().len(), step);
                Vec::new()
            }
            Action::Enter => {
                let to = self.enter();
                self.open(to)
            }
        }
    }

    /// Opens `to` when there is one, emptying the box, which the nav keeps from page to page.
    fn open(&mut self, to: Option<Destination>) -> Vec<Effect> {
        let Some(to) = to else {
            return Vec::new();
        };
        self.query.clear();
        self.go_idle();
        vec![Effect::Navigate(to)]
    }

    /// Where Enter goes: the lit result, else the results page while the box holds a name
    /// that hasn't been answered with nothing.
    fn enter(&self) -> Option<Destination> {
        if let Some(card) = self.highlight().and_then(|at| self.results().get(at)) {
            return Some(destination(card));
        }
        if self.status == Status::NoMatch {
            return None;
        }
        name_lookup(&self.query).map(|_| Destination::Results {
            query: self.query.trim().to_owned(),
        })
    }

    fn type_text(&mut self, text: String) -> Vec<Effect> {
        self.query = text;
        self.highlight = None;
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
        // A no-match line answered the old text, not this.
        if self.status == Status::NoMatch {
            self.status = Status::Idle;
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
pub(crate) fn limit() -> usize {
    usize::try_from(LOOKUP_LIMIT).unwrap_or(usize::MAX)
}

/// The lookup for `text` as a name, or `None` when the server would refuse it (empty once
/// trimmed, too long, holding a control character, or with no letter or digit).
pub(crate) fn name_lookup(text: &str) -> Option<HttpCardLookup> {
    let name = LookupName::try_from(text.to_owned()).ok()?;
    HttpCardLookup::try_new(name, Vec::new(), Vec::new(), Vec::new()).ok()
}

/// Where a picked card opens: its name's card page, or its printing's page for a name with no
/// slug.
pub(crate) fn destination(card: &Card) -> Destination {
    let name = &card.scryfall_data.name;
    match slug::card_slug(name) {
        Some(slug) => Destination::Card(slug),
        None => Destination::Printing {
            path: slug::printing_path(name, card.scryfall_data.id),
        },
    }
}
