//! The card search's results page, `/search?q=<text>`: every card the lookup matches for the
//! text, drawn in the card page's several-cards grid, each opening its card page.
//!
//! Enter in the card search opens it with several matches. It looks the text up on load, the
//! same lookup the bar sends but paged, [`LOOKUP_PAGE`] cards at a time, and an answer with a
//! single card opens that card's page in its place, so a link to a text only one card matches
//! lands on the card. Scrolling to the end of the list, or the keyboard cursor's End, asks for the
//! next page ([`Action::LoadMore`]) until a short page ends it. Text no card could have matches
//! nothing without asking.
//!
//! The model holds the text and what came back; the client sends the lookup, with the session's
//! token when one is held, and draws the grid.

#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::models::lookup::{LOOKUP_PAGE, MAX_LOOKUP_OFFSET};
use zwipe_core::http::contracts::card::HttpCardLookup;

use crate::generation::{Generation, Ticket};
use crate::screens::card_search::{self, destination, name_lookup};
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
    /// Back on the page for results it kept ([`State::kept`]): its loaded pages show again with
    /// nothing looked up, and the next page follows on from them.
    Resume(Kept),
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
    /// The list scrolled to its end, or the keyboard cursor's End: the next page, while the
    /// lookup may have more and no next page is out.
    LoadMore,
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

/// The results a page showed for one text, kept so Back from a card opens on them as they were:
/// every page loaded, and whether the lookup had more.
#[derive(Debug, Clone, PartialEq)]
pub struct Kept {
    query: String,
    cards: Vec<Card>,
    exhausted: bool,
}

impl Kept {
    /// The text the results matched.
    #[must_use]
    pub fn query(&self) -> &str {
        &self.query
    }
}

/// The page for one text.
#[derive(Debug, Clone, Default)]
pub struct State {
    query: String,
    status: Status,
    cards: Vec<Card>,
    tickets: Generation,
    /// Whether the last page was short, so the lookup has no more.
    exhausted: bool,
    /// The ticket of the next page out, if one is.
    more: Option<Ticket>,
    /// Where the newest next page's cards start, which deal in; `None` on a first page.
    dealt_from: Option<usize>,
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

    /// The matches while [`Status::Several`] shows, every page dealt so far.
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
                self.more = None;
                self.exhausted = false;
                self.dealt_from = None;
                self.lookup()
            }
            Action::Loaded { ticket, cards } if self.more == Some(ticket) => {
                self.more = None;
                self.dealt_from = Some(self.cards.len());
                self.exhausted = is_last_page(cards.len());
                let fresh: Vec<Card> = cards
                    .into_iter()
                    .filter(|card| {
                        !self
                            .cards
                            .iter()
                            .any(|held| held.scryfall_data.id == card.scryfall_data.id)
                    })
                    .collect();
                self.cards.extend(fresh);
                Vec::new()
            }
            Action::Loaded { ticket, cards } => {
                if !self.is_answer(ticket) {
                    return Vec::new();
                }
                self.exhausted = is_last_page(cards.len());
                self.cards = cards;
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
            Action::Failed { ticket, .. } if self.more == Some(ticket) => {
                self.more = None;
                Vec::new()
            }
            Action::LoadMore => self.load_more(),
            Action::Resume(kept) => {
                let _ = self.tickets.next_ticket();
                self.query = kept.query;
                self.cards = kept.cards;
                self.exhausted = kept.exhausted;
                self.more = None;
                self.dealt_from = None;
                self.status = Status::Several;
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
        vec![Effect::Lookup {
            ticket,
            lookup: lookup.paged(0),
        }]
    }

    /// Asks for the next page, from as many cards as the list holds, while it may have more and
    /// none is out.
    fn load_more(&mut self) -> Vec<Effect> {
        if !self.has_more() || self.more.is_some() {
            return Vec::new();
        }
        let Some(lookup) = name_lookup(&self.query) else {
            return Vec::new();
        };
        let Ok(offset) = u32::try_from(self.cards.len()) else {
            self.exhausted = true;
            return Vec::new();
        };
        if offset > MAX_LOOKUP_OFFSET {
            self.exhausted = true;
            return Vec::new();
        }
        let ticket = self.tickets.next_ticket();
        self.more = Some(ticket);
        vec![Effect::Lookup {
            ticket,
            lookup: lookup.paged(offset),
        }]
    }

    /// Whether the list may hold more past what it shows: it shows matches and the last page was
    /// full.
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.status == Status::Several && !self.exhausted
    }

    /// Where the newest next page's cards start in [`State::cards`], so they deal in as they
    /// land; `None` while the cards are a first page.
    #[must_use]
    pub fn dealt_from(&self) -> Option<usize> {
        self.dealt_from
    }

    /// The results to keep for Back from a card, while they show.
    #[must_use]
    pub fn kept(&self) -> Option<Kept> {
        (self.status == Status::Several).then(|| Kept {
            query: self.query.clone(),
            cards: self.cards.clone(),
            exhausted: self.exhausted,
        })
    }

    /// Whether a next page is out.
    #[must_use]
    pub fn loading_more(&self) -> bool {
        self.more.is_some()
    }
}

/// Whether a page of `got` cards was the lookup's last: shorter than a page.
fn is_last_page(got: usize) -> bool {
    u32::try_from(got).is_ok_and(|got| got < LOOKUP_PAGE)
}
