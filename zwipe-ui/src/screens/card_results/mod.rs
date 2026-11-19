//! The card search's results page, `/search?q=<text>`: every card the lookup matches for the
//! text, drawn in the card page's several-cards grid, each opening its card page.
//!
//! Enter in the card search opens it with several matches. It looks the text up once on load,
//! the same lookup the bar sends, and an answer with a single card opens that card's page in its
//! place, so a link to a text only one card matches lands on the card. Text no card could have
//! matches nothing without asking.
//!
//! The model holds the text and what came back; the client sends the lookup, with the session's
//! token when one is held, and draws the grid.

#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::HttpCardLookup;

use crate::generation::{Generation, Ticket};
use crate::screens::card_search::{self, destination, limit, name_lookup};
pub use crate::screens::card_search::{Destination, Failure};

/// The line when no card matches.
pub const NO_MATCH: &str = card_search::NO_MATCH;
/// The line while too many lookups wait, beside Retry.
pub const RATE_LIMITED: &str = "Too many searches, try again in a moment";
/// The line when the lookup failed, beside Retry.
pub const FAILED: &str = "Couldn't search";

/// What the page shows.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The lookup is out, or a single match is opening its card.
    #[default]
    Loading,
    /// The matches, in [`State::cards`].
    Several,
    /// No card matches.
    NoMatch,
    /// Too many lookups; [`Action::Retry`] asks again.
    RateLimited,
    /// The lookup failed; [`Action::Retry`] asks again.
    Failed,
}

/// Every load, tap and reply the page takes.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// The route's text, on mount and whenever the route changes it.
    Load(String),
    /// The lookup answered.
    Loaded {
        /// The [`Effect::Lookup`] ticket.
        ticket: Ticket,
        /// The cards it found.
        cards: Vec<Card>,
    },
    /// The lookup failed.
    Failed {
        /// The [`Effect::Lookup`] ticket.
        ticket: Ticket,
        /// Why.
        failure: Failure,
    },
    /// Retry after a failure.
    Retry,
    /// A card tapped, by its printing id.
    Pick(Uuid),
}

/// What an action asks the client to do.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Sends the lookup; answers with [`Action::Loaded`] or [`Action::Failed`] carrying `ticket`.
    Lookup {
        /// The ticket to hand back.
        ticket: Ticket,
        /// What to look up.
        lookup: HttpCardLookup,
    },
    /// Opens a card in place of this page, so Back skips the page.
    Replace(Destination),
    /// Opens a card.
    Navigate(Destination),
}

/// The page for one text.
#[derive(Debug, Clone, Default)]
pub struct State {
    query: String,
    status: Status,
    cards: Vec<Card>,
    tickets: Generation,
}

impl State {
    /// A page that hasn't loaded.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The text, trimmed.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// What the page shows.
    #[must_use]
    pub fn status(&self) -> Status {
        self.status
    }

    /// The matches while [`Status::Several`] shows, at most the lookup's limit.
    #[must_use]
    pub fn cards(&self) -> &[Card] {
        if self.status == Status::Several {
            &self.cards
        } else {
            &[]
        }
    }

    /// The page's heading and title.
    #[must_use]
    pub fn title(&self) -> String {
        format!("Cards matching “{}”", self.query)
    }

    /// Takes one action and answers with what the client should do.
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load(text) => {
                text.trim().clone_into(&mut self.query);
                self.cards.clear();
                self.lookup()
            }
            Action::Loaded { ticket, cards } => {
                if !self.is_answer(ticket) {
                    return Vec::new();
                }
                self.cards = cards.into_iter().take(limit()).collect();
                if let [only] = self.cards.as_slice() {
                    return vec![Effect::Replace(destination(only))];
                }
                self.status = if self.cards.is_empty() {
                    Status::NoMatch
                } else {
                    Status::Several
                };
                Vec::new()
            }
            Action::Failed { ticket, failure } => {
                if self.is_answer(ticket) {
                    self.status = match failure {
                        Failure::RateLimited { .. } => Status::RateLimited,
                        Failure::Other => Status::Failed,
                    };
                }
                Vec::new()
            }
            Action::Retry => match self.status {
                Status::Failed | Status::RateLimited => self.lookup(),
                Status::Loading | Status::Several | Status::NoMatch => Vec::new(),
            },
            Action::Pick(id) => self
                .cards()
                .iter()
                .find(|card| card.scryfall_data.id == id)
                .map(|card| Effect::Navigate(destination(card)))
                .into_iter()
                .collect(),
        }
    }

    /// Whether `ticket`'s reply is the one the page waits for.
    fn is_answer(&self, ticket: Ticket) -> bool {
        self.tickets.is_current(ticket) && self.status == Status::Loading
    }

    /// Sends the lookup for the text under a new ticket, or shows no match for text no card
    /// could have.
    fn lookup(&mut self) -> Vec<Effect> {
        let ticket = self.tickets.next_ticket();
        let Some(lookup) = name_lookup(&self.query) else {
            self.status = Status::NoMatch;
            return Vec::new();
        };
        self.status = Status::Loading;
        vec![Effect::Lookup { ticket, lookup }]
    }
}
