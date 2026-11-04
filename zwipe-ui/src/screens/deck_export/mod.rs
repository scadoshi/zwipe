//! Export: a deck as a decklist that Import reads back, with chips for the boards it includes and Copy.
//!
//! The client keeps a [`State`] while the screen is up and sends [`Action::Load`] with the deck's id when it opens. The decklist comes from [`crate::deck::export::text`] and the at-least-one rule from [`Boards`]. The read-only text box and the clipboard stay in the client; Copy asks for both through [`Effect::Copy`].

#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::deck::Deck;
use zwipe_core::domain::user::hints::OneTimeHint;

use crate::deck::export::{self, Boards};
use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// The screen's title.
pub const TITLE: &str = "Export";

/// The label on the chip row that picks the boards to include.
pub const BOARDS: &str = "Export";

/// The label over the decklist.
pub const DECKLIST: &str = "Decklist";

/// The line in place of the decklist when the deck failed to load.
pub const LOAD_FAILED: &str = "Could not load deck";

/// The Back button, to the deck.
pub const BACK: &str = "Back";

/// The button that copies the decklist.
pub const COPY: &str = "Copy";

/// The toast after Copy.
pub const COPIED: &str = "Copied to clipboard";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The deck view for this deck, from Back.
    Deck(Uuid),
    /// The deck list, in place of a screen whose deck is gone.
    DeckList,
}

/// One board chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Board {
    /// The main deck.
    Deck,
    /// The maybeboard.
    Maybe,
    /// The sideboard.
    Side,
}

impl Board {
    /// Every board, in chip order.
    pub const ALL: [Self; 3] = [Self::Deck, Self::Maybe, Self::Side];

    /// The chip's words.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Deck => "Deck",
            Self::Maybe => "Maybe",
            Self::Side => "Side",
        }
    }
}

/// Where the deck fetch stands, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The fetch is out: the chip row and the decklist ghost.
    Loading,
    /// The fetch failed: the chips over [`LOAD_FAILED`].
    Failed,
    /// The chips over the decklist, which is empty for a deck with no cards.
    Loaded,
}

/// What a user does on Export, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Fetches this deck, as the screen does when it opens. The boards stay as they are.
    Load(Uuid),
    /// The deck arrived for the fetch holding `ticket`.
    Loaded {
        /// The ticket from the [`Effect::FetchDeck`] this answers.
        ticket: Ticket,
        /// The deck.
        deck: Box<Deck>,
    },
    /// The fetch holding `ticket` failed. The client has already reported it and raised its error toast.
    Failed {
        /// The ticket from the [`Effect::FetchDeck`] this answers.
        ticket: Ticket,
    },
    /// The request holding `ticket` found no deck: the server no longer has it. The client has already reported it and raised its toast.
    Gone {
        /// The ticket from the [`Effect::FetchDeck`] this answers.
        ticket: Ticket,
    },
    /// Turns a board on or off under the at-least-one rule.
    Toggle(Board),
    /// Copies the decklist. Does nothing until the deck has loaded.
    Copy,
    /// Goes back to the deck.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches the deck; answers with [`Action::Loaded`] or [`Action::Failed`] carrying `ticket`.
    FetchDeck {
        /// The ticket to hand back.
        ticket: Ticket,
        /// The deck to fetch.
        deck_id: Uuid,
    },
    /// Copies `text` to the clipboard, then raises `done`.
    Copy {
        /// The decklist.
        text: String,
        /// The toast after the copy.
        done: Toast,
    },
    /// Goes to this screen.
    Navigate(Destination),
    /// Goes to this screen in place of the current one, so Back skips a screen with nothing left to show.
    Replace(Destination),
}

/// The deck as the last fetch left it.
#[derive(Debug, Default, Clone, PartialEq)]
enum Load {
    #[default]
    Loading,
    Failed,
    Loaded(Box<Deck>),
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// Export for one visit. Starts loading with only the deck board on; send [`Action::Load`] to start the fetch.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    deck_id: Option<Uuid>,
    load: Load,
    boards: Boards,
    hint: HintOffer,
    tickets: Generation,
}

impl State {
    /// The screen as it first draws: loading, only the deck board on.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// A reply whose ticket a newer [`Action::Load`] has replaced changes nothing. The hint is offered once per visit, with the first fetch.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load(deck_id) => {
                self.deck_id = Some(deck_id);
                self.load = Load::Loading;
                let mut effects = Vec::new();
                if self.hint == HintOffer::Waiting {
                    self.hint = HintOffer::Offered;
                    effects.push(Effect::OfferHint(OneTimeHint::Export));
                }
                effects.push(Effect::FetchDeck {
                    ticket: self.tickets.next_ticket(),
                    deck_id,
                });
                return effects;
            }
            Action::Loaded { ticket, deck } => {
                if self.tickets.is_current(ticket) {
                    self.load = Load::Loaded(deck);
                }
            }
            Action::Failed { ticket } => {
                if self.tickets.is_current(ticket) {
                    self.load = Load::Failed;
                }
            }
            Action::Gone { ticket } => {
                if self.tickets.is_current(ticket) {
                    self.load = Load::Failed;
                }
                return vec![Effect::Replace(Destination::DeckList)];
            }
            Action::Toggle(board) => match board {
                Board::Deck => self.boards.toggle_deck(),
                Board::Maybe => self.boards.toggle_maybe(),
                Board::Side => self.boards.toggle_side(),
            },
            Action::Copy => {
                return self
                    .text()
                    .map(|text| Effect::Copy {
                        text,
                        done: Toast::quick(ToastKind::Info, COPIED),
                    })
                    .into_iter()
                    .collect();
            }
            Action::Back => {
                return self
                    .deck_id
                    .map(|id| Effect::Navigate(Destination::Deck(id)))
                    .into_iter()
                    .collect();
            }
        }
        Vec::new()
    }

    /// The deck, once [`Action::Load`] has named it.
    #[must_use]
    pub fn deck_id(&self) -> Option<Uuid> {
        self.deck_id
    }

    /// Where the fetch stands.
    #[must_use]
    pub fn status(&self) -> Status {
        match self.load {
            Load::Loading => Status::Loading,
            Load::Failed => Status::Failed,
            Load::Loaded(_) => Status::Loaded,
        }
    }

    /// The boards the decklist includes. At least one is always on.
    #[must_use]
    pub fn boards(&self) -> Boards {
        self.boards
    }

    /// Whether `board`'s chip is on.
    #[must_use]
    pub fn is_on(&self, board: Board) -> bool {
        match board {
            Board::Deck => self.boards.deck,
            Board::Maybe => self.boards.maybe,
            Board::Side => self.boards.side,
        }
    }

    /// The decklist for the boards on, once the deck has loaded.
    #[must_use]
    pub fn text(&self) -> Option<String> {
        match &self.load {
            Load::Loaded(deck) => Some(export::text(deck, self.boards)),
            Load::Loading | Load::Failed => None,
        }
    }
}
