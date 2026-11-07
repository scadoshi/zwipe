//! Remove cards: swipe through one deck's cards to trim it.
//!
//! Left keeps the card and cycles past it, right removes every copy, up moves it between the main board and the maybeboard, and down undoes the last swipe. The Boards chips pick which board is trimmed, and the client's card filter narrows and sorts what shows. The stack cycles: a kept card comes round again, a removed or moved one leaves it.
//!
//! The model holds the deck's entries, the stack and its cursor, the board pick, the land and budget crossings and every toast. The undo log the deck cards view walks belongs to the client, so the model asks for an entry back with [`Effect::TakeUndo`] and records one with [`Effect::RecordUndo`]. Gestures, the card's exit and entrance motion, the filter sheet, the details and printings sheets stay in the client.

#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::{Board, Deck, DeckCard, DeckEntry, deck_metrics};
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};

use crate::deck_signals::{self, Crossing, Level};
use crate::generation::{Generation, Ticket};
use crate::stack::{CardStack, Direction, NO_CARDS_LEFT, RemoveAction, StackAction};
use crate::stores::undo::{self, UndoAction};
use crate::toast::{Line, Toast, ToastKind, keep_one};

/// The screen's title.
pub const TITLE: &str = "Remove";

/// The hint's title.
pub const HINT_TITLE: &str = "Swipe to trim";

/// The label in front of the board chips.
pub const BOARDS: &str = "Boards";

/// The Back button, to where the user came from.
pub const BACK: &str = "Back";

/// The button that opens the card filter.
pub const FILTER: &str = "Filter";

/// The button that brings the stack back to its first card.
pub const REFRESH: &str = "Refresh";

/// The dock's undo button, which does what a down swipe does.
pub const UNDO: &str = "Undo";

/// The toast for a left swipe.
pub const KEPT: &str = "Kept";

/// The toast for a right swipe.
pub const REMOVED: &str = "Removed from deck";

/// The toast for an up swipe that moves the card to the maybeboard.
pub const MOVED_TO_MAYBEBOARD: &str = "Moved to maybeboard";

/// The toast for an up swipe that moves a maybeboard card to the main board.
pub const MOVED_TO_MAIN: &str = "Moved to main";

/// The toast for an undo with no swipe to undo.
pub const NOTHING_TO_UNDO: &str = "Nothing to undo";

/// The toast for undoing a keep.
pub const UNDID_KEEP: &str = "Undid keep";

/// The toast once an undone remove lands.
pub const UNDID_REMOVE: &str = "Undid remove";

/// The toast once an undone move lands.
pub const UNDID_MOVE: &str = "Undid move";

/// The toast for undoing a swipe the deck cards view's Undo already reversed.
pub const ALREADY_CHANGED: &str = "Already changed";

/// The toast for Refresh.
pub const STACK_REFRESHED: &str = "Stack refreshed";

/// What a swipe says of itself, which the run-out toast replaces.
const SWIPE_LINES: [&str; 4] = [KEPT, REMOVED, MOVED_TO_MAYBEBOARD, MOVED_TO_MAIN];

/// The toast when the picked boards hold no cards.
pub const NO_CARDS_ON_BOARDS: &str = "No cards on these boards";

/// The toast when the filter matches no card on the picked boards.
pub const NO_MATCHES: &str = "No cards match your filters";

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// Wherever the user came from, from Back.
    Previous,
    /// The deck list, in place of a screen whose deck is gone.
    DeckList,
}

/// Which board the stack trims.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardFilter {
    /// The main board, the pick on entry.
    #[default]
    Deck,
    /// The maybeboard.
    Maybeboard,
    /// The sideboard.
    Sideboard,
    /// Every board.
    All,
}

impl BoardFilter {
    /// The chips in the order they draw.
    pub const ALL: [Self; 4] = [Self::Deck, Self::Maybeboard, Self::Sideboard, Self::All];

    /// The chip's words.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Deck => "Main",
            Self::Maybeboard => "Maybe",
            Self::Sideboard => "Side",
            Self::All => "All",
        }
    }

    /// Whether a card on `board` shows under this pick.
    #[must_use]
    pub fn holds(self, board: Board) -> bool {
        match self {
            Self::Deck => board.is_active(),
            Self::Maybeboard => board.is_maybeboard(),
            Self::Sideboard => board.is_sideboard(),
            Self::All => true,
        }
    }
}

/// Where the screen stands, for the client to pick what it draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The deck is still loading.
    Loading,
    /// The deck failed to load.
    Failed,
    /// The deck loaded and no card shows under the board and filter.
    Empty,
    /// Cards show: [`State::window`] and [`State::current`].
    Loaded,
}

/// The undo log entry an undo asks the client to take back, found by the swipe it reverses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UndoClaim {
    /// The entry recorded when this card was removed.
    Removed {
        /// The removed card's printing ID.
        card_id: Uuid,
    },
    /// The entry recorded when this card moved off `from`.
    MovedBoard {
        /// The moved card's printing ID.
        card_id: Uuid,
        /// The board it left.
        from: Board,
    },
}

impl UndoClaim {
    /// Whether `action` is the entry this claim takes, for [`crate::stores::undo::UndoStore::take_newest`].
    #[must_use]
    pub fn matches(&self, action: &UndoAction) -> bool {
        match (self, action) {
            (Self::Removed { card_id }, UndoAction::Removed { entry, .. }) => {
                entry.card.scryfall_data.id == *card_id
            }
            (
                Self::MovedBoard { card_id, from },
                UndoAction::MovedBoard {
                    card_id: moved,
                    from: left,
                    ..
                },
            ) => moved == card_id && left == from,
            _ => false,
        }
    }
}

/// What a user does on Remove cards, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Opens the screen for `deck_id` with the filter the client restored, offering the hint once per visit.
    Load {
        /// The deck being trimmed.
        deck_id: Uuid,
        /// The card filter as the screen opens.
        filter: CardQueryBuilder,
    },
    /// The deck arrived for the fetch holding `ticket`.
    Loaded {
        /// The ticket from the [`Effect::FetchDeck`] this answers.
        ticket: Ticket,
        /// The deck.
        deck: Box<Deck>,
    },
    /// The request holding `ticket` failed. The client has already reported it and raised its error toast.
    Failed {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// The request holding `ticket` found no deck: the server no longer has it. The client has already reported it and raised its toast.
    Gone {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// The client's card filter was applied: the stack is built again from the first card.
    Filter(CardQueryBuilder),
    /// Picks the board to trim; the stack is built again from the first card.
    PickBoard(BoardFilter),
    /// Keeps the card on top and cycles past it, as its left exit commits.
    Keep,
    /// Removes every copy of the card on top, as its right exit commits.
    Remove,
    /// Moves the card on top between the main board and the maybeboard, as its up exit commits. A sideboard card goes to the maybeboard.
    Move,
    /// Undoes the newest swipe.
    Undo,
    /// The client took the entry an [`Effect::TakeUndo`] asked for, or found none.
    Taken {
        /// The ticket from the [`Effect::TakeUndo`] this answers.
        ticket: Ticket,
        /// The entry taken off the undo log.
        taken: Option<UndoAction>,
    },
    /// The request holding `ticket` went through: [`Effect::DeleteDeckCard`], [`Effect::MoveDeckCard`] or [`Effect::RestoreBoard`].
    Landed {
        /// The ticket from the request this answers.
        ticket: Ticket,
    },
    /// The [`Effect::RestoreDeckCard`] holding `ticket` put the card back.
    Restored {
        /// The ticket from the [`Effect::RestoreDeckCard`] this answers.
        ticket: Ticket,
        /// The deck card the server created.
        deck_card: Box<DeckCard>,
    },
    /// Brings the stack back to its first card and forgets the swipes; a Random sort deals again.
    Refresh,
    /// Goes back to where the user came from.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches the deck and answers with [`Action::Loaded`] or [`Action::Failed`].
    FetchDeck {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck to fetch.
        deck_id: Uuid,
    },
    /// Deletes every copy of the card and answers with [`Action::Landed`] or [`Action::Failed`].
    DeleteDeckCard {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The card's printing ID.
        card_id: Uuid,
    },
    /// Sends the card to `to` with [`board_request`] and answers with [`Action::Landed`] or [`Action::Failed`].
    MoveDeckCard {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The card's printing ID.
        card_id: Uuid,
        /// The board it goes to.
        to: Board,
    },
    /// Takes the newest undo log entry `claim` matches off the deck's log and answers at once with [`Action::Taken`].
    TakeUndo {
        /// The ticket to hand back with the answer.
        ticket: Ticket,
        /// The deck whose log to search.
        deck_id: Uuid,
        /// Which entry to take.
        claim: UndoClaim,
    },
    /// Undoes a remove: creates the card again with [`restore_request`] and answers with [`Action::Restored`] or [`Action::Failed`]. A failure here is the undo's own: the client words it as one.
    RestoreDeckCard {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The card to put back.
        card: Box<Card>,
        /// The board it sat on.
        board: Board,
        /// The copies it had.
        quantity: i32,
    },
    /// Undoes a move: sends the card back to `board` with [`board_request`] and answers with [`Action::Landed`] or [`Action::Failed`]. A failure here is the undo's own: the client words it as one.
    RestoreBoard {
        /// The ticket to hand back with the reply.
        ticket: Ticket,
        /// The deck.
        deck_id: Uuid,
        /// The card's printing ID.
        card_id: Uuid,
        /// The board it left.
        board: Board,
    },
    /// Pushes `action` onto the deck's undo log.
    RecordUndo {
        /// The deck whose log it joins.
        deck_id: Uuid,
        /// The entry.
        action: UndoAction,
    },
    /// Plays the top card's entrance from this side: an undo brought it back.
    Enter(Direction),
    /// Goes to this screen.
    Navigate(Destination),
    /// Goes to this screen in place of the current one, so Back skips a screen with nothing left to show.
    Replace(Destination),
    /// Raises this toast.
    Toast(Toast),
    /// Confirms what a swipe or its undo just did, as the user's swipe
    /// confirmations setting says: this toast's words in the screen's chip,
    /// the toast itself, or nothing.
    Confirm(Toast),
}

/// Which of an action's effects raise a toast, for [`keep_one`].
fn toast_line(effect: &Effect) -> Option<Line<'_>> {
    match effect {
        Effect::Toast(toast) => Some(Line::Toast(toast)),
        Effect::Confirm(toast) => Some(Line::Confirm(toast)),
        _ => None,
    }
}

/// The create request that puts `quantity` copies of `card` back on `board`.
#[must_use]
pub fn restore_request(card: &Card, board: Board, quantity: i32) -> HttpCreateDeckCard {
    HttpCreateDeckCard::new(
        &card.scryfall_data,
        quantity,
        undo::create_board_field(board),
    )
}

/// The update request that moves a card to `board`.
#[must_use]
pub fn board_request(board: Board) -> HttpPatchDeckCard {
    HttpPatchDeckCard::new(None, Some(board.display_name().to_owned()))
}

/// Whether the visit has offered the one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// Where the deck's fetch stands.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum Fetch {
    #[default]
    Loading,
    Failed,
    Loaded,
}

/// A request out, waiting for its reply.
#[derive(Debug, Clone, PartialEq)]
enum Pending {
    /// The deck's fetch.
    Load,
    /// A remove of `card_id`. The entry, when the deck held it, goes on the undo log once it lands. `undo` holds an undo asked for while the request was out: the card's restore goes out once the remove lands, so it never races the delete.
    Remove {
        card_id: Uuid,
        entry: Option<Box<DeckEntry>>,
        undo: Option<HeldUndo>,
    },
    /// A move. The board it left, when the deck held it, goes on the undo log once it lands.
    Move {
        card_id: Uuid,
        card_name: String,
        from: Option<Board>,
    },
    /// An undo waiting for the client to take its log entry.
    Claim(RemoveAction),
    /// An undone remove. `taken` goes back on the log if it fails.
    UndoRemove {
        card: Box<Card>,
        taken: Option<UndoAction>,
    },
    /// An undone move. `taken` goes back on the log if it fails.
    UndoMove {
        card_id: Uuid,
        from: Board,
        taken: Option<UndoAction>,
    },
}

/// An undone remove waiting on its own delete: what the restore puts back.
#[derive(Debug, Clone, PartialEq)]
struct HeldUndo {
    card: Box<Card>,
    board: Board,
    quantity: i32,
}

/// Remove cards for one visit. Starts loading with no card; send [`Action::Load`] with the deck.
///
/// Every request takes a fresh [`Ticket`] from one [`Generation`] and waits in a list keyed by it, so each reply finds its own request. A new [`Action::Load`] empties the list, so a reply from before it changes nothing. Removes and moves change the stack and the entries at once; their replies only record the undo log entry.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    deck_id: Option<Uuid>,
    fetch: Fetch,
    filter: CardQueryBuilder,
    board: BoardFilter,
    entries: Vec<DeckEntry>,
    command_zone_cards: Vec<Card>,
    /// The deck's own land target; no land toast without one.
    land_target: Option<i32>,
    budget: Option<f64>,
    currency: PriceCurrency,
    stack: CardStack<RemoveAction>,
    waiting: Vec<(Ticket, Pending)>,
    generation: Generation,
    hint: HintOffer,
}

impl State {
    /// The screen as it first draws: loading, no card, the main board picked.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order. When the pile turns empty it says so with a toast; a swipe that empties it says only that, not its own line.
    ///
    /// A reply whose request is no longer waiting changes nothing.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        let swiped = matches!(action, Action::Keep | Action::Remove | Action::Move);
        let before = self.status();
        let mut effects = self.apply(action);
        if before != Status::Empty && self.status() == Status::Empty {
            let line = if swiped {
                effects.retain(|effect| {
                    !matches!(effect, Effect::Toast(toast) | Effect::Confirm(toast) if SWIPE_LINES.contains(&toast.text.as_str()))
                });
                NO_CARDS_LEFT
            } else if self.filter.is_empty() {
                NO_CARDS_ON_BOARDS
            } else {
                NO_MATCHES
            };
            effects.push(Effect::Toast(Toast::quick(ToastKind::Info, line)));
        }
        keep_one(&mut effects, toast_line);
        effects
    }

    fn apply(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load { deck_id, filter } => self.load(deck_id, filter),
            Action::Loaded { ticket, deck } => {
                if matches!(self.take(ticket), Some(Pending::Load)) {
                    self.land_deck(*deck);
                }
                Vec::new()
            }
            Action::Failed { ticket } => self.failed(ticket),
            Action::Gone { ticket } => self.gone(ticket),
            Action::Filter(filter) => {
                self.filter = filter;
                self.rerun();
                Vec::new()
            }
            Action::PickBoard(board) => {
                self.board = board;
                self.rerun();
                Vec::new()
            }
            Action::Keep => self.keep(),
            Action::Remove => self.remove(),
            Action::Move => self.move_board(),
            Action::Undo => self.undo(),
            Action::Taken { ticket, taken } => self.taken(ticket, taken),
            Action::Landed { ticket } => self.landed(ticket),
            Action::Restored { ticket, deck_card } => self.restored(ticket, *deck_card),
            Action::Refresh => {
                self.stack.rewind();
                if self.filter.sort() == Some(CardSortKey::Random) {
                    self.rerun();
                }
                vec![Effect::Toast(Toast::quick(
                    ToastKind::Info,
                    STACK_REFRESHED,
                ))]
            }
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
        }
    }

    /// Starts the visit for `deck_id`, keeping the generation so replies from an earlier visit stay stale.
    fn load(&mut self, deck_id: Uuid, filter: CardQueryBuilder) -> Vec<Effect> {
        let hint = self.hint;
        *self = Self {
            deck_id: Some(deck_id),
            filter,
            hint: HintOffer::Offered,
            generation: self.generation,
            ..Self::default()
        };
        let mut effects = Vec::new();
        if hint == HintOffer::Waiting {
            effects.push(Effect::OfferHint(OneTimeHint::RemoveDeckCards));
        }
        let ticket = self.wait(Pending::Load);
        effects.push(Effect::FetchDeck { ticket, deck_id });
        effects
    }

    /// Takes in the deck and builds the stack.
    fn land_deck(&mut self, deck: Deck) {
        let profile = deck.deck_profile;
        self.land_target = profile.land_target;
        self.budget = profile.price_target;
        self.currency = profile.price_target_currency.unwrap_or(PriceCurrency::Usd);
        self.command_zone_cards = deck.command_zone_cards;
        self.entries = deck.entries;
        self.fetch = Fetch::Loaded;
        self.rerun();
    }

    /// Settles a failed request.
    fn gone(&mut self, ticket: Ticket) -> Vec<Effect> {
        let mut effects = self.failed(ticket);
        effects.push(Effect::Replace(Destination::DeckList));
        effects
    }

    fn failed(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        match self.take(ticket) {
            Some(Pending::Load) => {
                self.fetch = Fetch::Failed;
                Vec::new()
            }
            // The delete failed, so the card never left the deck: an undo held
            // on it only puts the entry back.
            Some(Pending::Remove {
                entry,
                undo: Some(_),
                ..
            }) => {
                if let Some(entry) = entry {
                    self.entries.push(*entry);
                }
                vec![Effect::Confirm(Toast::quick(
                    ToastKind::Success,
                    UNDID_REMOVE,
                ))]
            }
            Some(
                Pending::UndoRemove {
                    taken: Some(action),
                    ..
                }
                | Pending::UndoMove {
                    taken: Some(action),
                    ..
                },
            ) => vec![Effect::RecordUndo { deck_id, action }],
            _ => Vec::new(),
        }
    }

    /// Builds the stack from the picked board, then the filter and its sort, from the first card. The swipes stay for undo. Waits for the deck.
    fn rerun(&mut self) {
        if self.fetch != Fetch::Loaded {
            return;
        }
        let board = self.board;
        let on_board: Vec<Card> = self
            .entries
            .iter()
            .filter(|e| board.holds(e.deck_card.board))
            .map(|e| e.card.clone())
            .collect();
        let matching: Vec<Card> = if self.filter.is_empty() {
            on_board
        } else {
            match self.filter.build_criteria() {
                Ok(criteria) => Cards::from(on_board).matching(&criteria).into(),
                Err(_) => on_board,
            }
        };
        let sorted: Vec<Card> = Cards::from(matching)
            .sorted(self.filter.sort(), self.filter.direction())
            .into();
        self.stack.replace(sorted);
    }

    /// Keeps the top card and cycles past it.
    fn keep(&mut self) -> Vec<Effect> {
        if self.stack.current_wrapping().is_none() {
            return Vec::new();
        }
        self.stack.record(RemoveAction::Keep);
        self.stack.advance_wrapping();
        vec![Effect::Confirm(Toast::quick(ToastKind::Info, KEPT))]
    }

    /// Removes every copy of the top card from the stack and the entries, and asks the server to.
    fn remove(&mut self) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let Some(card) = self.stack.current_wrapping().cloned() else {
            return Vec::new();
        };
        let card_id = card.scryfall_data.id;
        let entry = self.entry(card_id).cloned();
        let (board, quantity) = entry.as_ref().map_or((Board::Deck, 1), |e| {
            (e.deck_card.board, *e.deck_card.quantity)
        });
        self.stack.record(RemoveAction::Remove {
            card: Box::new(card),
            board,
            quantity,
        });
        let ticket = self.wait(Pending::Remove {
            card_id,
            entry: entry.map(Box::new),
            undo: None,
        });
        let mut effects = vec![
            Effect::DeleteDeckCard {
                ticket,
                deck_id,
                card_id,
            },
            Effect::Confirm(Toast::quick(ToastKind::Success, REMOVED)),
        ];
        let (lands, price) = (self.land_count(), self.price());
        self.entries.retain(|e| e.card.scryfall_data.id != card_id);
        self.stack.remove_current();
        effects.extend(self.crossings(lands, price));
        effects
    }

    /// Moves the top card between the main board and the maybeboard, dropping it from the stack, and asks the server to.
    fn move_board(&mut self) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let Some(card) = self.stack.current_wrapping().cloned() else {
            return Vec::new();
        };
        let card_id = card.scryfall_data.id;
        let card_name = card.scryfall_data.name.clone();
        let held = self.entry(card_id).map(|e| e.deck_card.board);
        let from = held.unwrap_or_default();
        let to = if from.is_maybeboard() {
            Board::Deck
        } else {
            Board::Maybeboard
        };
        self.stack.record(RemoveAction::MoveBoard {
            card: Box::new(card),
            from,
            to,
        });
        let ticket = self.wait(Pending::Move {
            card_id,
            card_name,
            from: held,
        });
        let message = if to.is_maybeboard() {
            MOVED_TO_MAYBEBOARD
        } else {
            MOVED_TO_MAIN
        };
        let mut effects = vec![
            Effect::MoveDeckCard {
                ticket,
                deck_id,
                card_id,
                to,
            },
            Effect::Confirm(Toast::quick(ToastKind::Info, message)),
        ];
        let (lands, price) = (self.land_count(), self.price());
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.card.scryfall_data.id == card_id)
        {
            entry.deck_card.board = to;
        }
        self.stack.remove_current();
        effects.extend(self.crossings(lands, price));
        effects
    }

    /// Undoes the newest swipe. A keep steps back at once; a remove or a move first asks the client for its undo log entry.
    fn undo(&mut self) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let Some(action) = self.stack.pop_action() else {
            return vec![Effect::Toast(Toast::normal(
                ToastKind::Info,
                NOTHING_TO_UNDO,
            ))];
        };
        let claim = match &action {
            RemoveAction::Keep => {
                if !self.stack.retreat_wrapping() {
                    return Vec::new();
                }
                return vec![
                    Effect::Enter(action.exited()),
                    Effect::Confirm(Toast::quick(ToastKind::Info, UNDID_KEEP)),
                ];
            }
            RemoveAction::Remove {
                card,
                board,
                quantity,
            } => {
                let card_id = card.scryfall_data.id;
                if let Some(held) = self.held_remove(card_id) {
                    // The delete is still out: the card comes back now and its
                    // restore waits for the delete to land.
                    *held = Some(HeldUndo {
                        card: card.clone(),
                        board: *board,
                        quantity: *quantity,
                    });
                    self.stack.insert_current((**card).clone());
                    return vec![Effect::Enter(action.exited())];
                }
                UndoClaim::Removed { card_id }
            }
            RemoveAction::MoveBoard { card, from, .. } => UndoClaim::MovedBoard {
                card_id: card.scryfall_data.id,
                from: *from,
            },
        };
        let ticket = self.wait(Pending::Claim(action));
        vec![Effect::TakeUndo {
            ticket,
            deck_id,
            claim,
        }]
    }

    /// Goes on with an undo once the client has taken its log entry. With no entry and the swipe already reversed, the deck cards view's Undo got there first and nothing is sent.
    fn taken(&mut self, ticket: Ticket, taken: Option<UndoAction>) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        let Some(Pending::Claim(action)) = self.take(ticket) else {
            return Vec::new();
        };
        let direction = action.exited();
        match action {
            RemoveAction::Keep => Vec::new(),
            RemoveAction::Remove {
                card,
                board,
                quantity,
            } => {
                let card_id = card.scryfall_data.id;
                if taken.is_none() && self.entry(card_id).is_some() {
                    return already_changed();
                }
                self.stack.insert_current((*card).clone());
                let ticket = self.wait(Pending::UndoRemove {
                    card: card.clone(),
                    taken,
                });
                vec![
                    Effect::Enter(direction),
                    Effect::RestoreDeckCard {
                        ticket,
                        deck_id,
                        card,
                        board,
                        quantity,
                    },
                ]
            }
            RemoveAction::MoveBoard { card, from, .. } => {
                let card_id = card.scryfall_data.id;
                let back = self
                    .entry(card_id)
                    .is_some_and(|e| e.deck_card.board == from);
                if taken.is_none() && back {
                    return already_changed();
                }
                self.stack.insert_current(*card);
                let ticket = self.wait(Pending::UndoMove {
                    card_id,
                    from,
                    taken,
                });
                vec![
                    Effect::Enter(direction),
                    Effect::RestoreBoard {
                        ticket,
                        deck_id,
                        card_id,
                        board: from,
                    },
                ]
            }
        }
    }

    /// Settles a request that went through with no body.
    fn landed(&mut self, ticket: Ticket) -> Vec<Effect> {
        let Some(deck_id) = self.deck_id else {
            return Vec::new();
        };
        match self.take(ticket) {
            Some(Pending::Remove {
                undo: Some(held), ..
            }) => {
                let HeldUndo {
                    card,
                    board,
                    quantity,
                } = held;
                let ticket = self.wait(Pending::UndoRemove {
                    card: card.clone(),
                    taken: None,
                });
                vec![Effect::RestoreDeckCard {
                    ticket,
                    deck_id,
                    card,
                    board,
                    quantity,
                }]
            }
            Some(Pending::Remove {
                entry: Some(entry), ..
            }) => {
                let baseline = *entry.deck_card.quantity;
                vec![Effect::RecordUndo {
                    deck_id,
                    action: UndoAction::Removed { entry, baseline },
                }]
            }
            Some(Pending::Move {
                card_id,
                card_name,
                from: Some(from),
            }) => vec![Effect::RecordUndo {
                deck_id,
                action: UndoAction::MovedBoard {
                    card_id,
                    card_name,
                    from,
                },
            }],
            Some(Pending::UndoMove { card_id, from, .. }) => {
                if let Some(entry) = self
                    .entries
                    .iter_mut()
                    .find(|e| e.card.scryfall_data.id == card_id)
                {
                    entry.deck_card.board = from;
                }
                vec![Effect::Confirm(Toast::quick(
                    ToastKind::Success,
                    UNDID_MOVE,
                ))]
            }
            _ => Vec::new(),
        }
    }

    /// Puts an undone remove's card back in the entries.
    fn restored(&mut self, ticket: Ticket, deck_card: DeckCard) -> Vec<Effect> {
        let Some(Pending::UndoRemove { card, .. }) = self.take(ticket) else {
            return Vec::new();
        };
        self.entries.push(DeckEntry {
            card: *card,
            deck_card,
        });
        vec![Effect::Confirm(Toast::quick(
            ToastKind::Success,
            UNDID_REMOVE,
        ))]
    }

    /// The land and budget toasts for the change from `lands` lands and a `price` total.
    fn crossings(&self, lands: i32, price: f64) -> Vec<Effect> {
        let land =
            deck_signals::land_crossing(self.land_target, lands, self.land_count(), Crossing::Down);
        let budget = deck_signals::budget_crossing(self.budget, self.currency, price, self.price());
        [land, budget]
            .into_iter()
            .flatten()
            .map(|(level, text)| Effect::Toast(Toast::normal(kind_of(level), text)))
            .collect()
    }

    /// Lands on the main board, counting copies.
    fn land_count(&self) -> i32 {
        deck_signals::land_count(&self.entries)
    }

    /// The main board and command zone's total in the budget's currency.
    fn price(&self) -> f64 {
        deck_metrics::deck_price(&self.entries, &self.command_zone_cards, self.currency)
    }

    /// The deck's entry for the printing `card_id`.
    fn entry(&self, card_id: Uuid) -> Option<&DeckEntry> {
        self.entries
            .iter()
            .find(|e| e.card.scryfall_data.id == card_id)
    }

    /// Takes a fresh ticket for `pending` and puts it on the waiting list.
    fn wait(&mut self, pending: Pending) -> Ticket {
        let ticket = self.generation.next_ticket();
        self.waiting.push((ticket, pending));
        ticket
    }

    /// The held-undo slot of a remove of `card_id` still out with no undo held yet, if there is one.
    fn held_remove(&mut self, card_id: Uuid) -> Option<&mut Option<HeldUndo>> {
        self.waiting
            .iter_mut()
            .rev()
            .find_map(|(_, pending)| match pending {
                Pending::Remove {
                    card_id: id,
                    undo: undo @ None,
                    ..
                } if *id == card_id => Some(undo),
                _ => None,
            })
    }

    /// Takes the request holding `ticket` off the waiting list, if it is there.
    fn take(&mut self, ticket: Ticket) -> Option<Pending> {
        let at = self.waiting.iter().position(|(t, _)| *t == ticket)?;
        Some(self.waiting.remove(at).1)
    }

    /// Where the screen stands.
    #[must_use]
    pub fn status(&self) -> Status {
        match self.fetch {
            Fetch::Loading => Status::Loading,
            Fetch::Failed => Status::Failed,
            Fetch::Loaded if self.stack.is_empty() => Status::Empty,
            Fetch::Loaded => Status::Loaded,
        }
    }

    /// The deck being trimmed, once [`Action::Load`] has named it.
    #[must_use]
    pub fn deck_id(&self) -> Option<Uuid> {
        self.deck_id
    }

    /// The picked board.
    #[must_use]
    pub fn board(&self) -> BoardFilter {
        self.board
    }

    /// The card filter the stack was last built with.
    #[must_use]
    pub fn filter(&self) -> &CardQueryBuilder {
        &self.filter
    }

    /// The deck's entries as the screen's swipes have left them.
    #[must_use]
    pub fn entries(&self) -> &[DeckEntry] {
        &self.entries
    }

    /// Whether the stack holds no card.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Whether a swipe this visit can still be undone.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.stack.has_history()
    }

    /// The card the details sheet and the card info describe, wrapping round the stack; `None` only when it is empty.
    #[must_use]
    pub fn current(&self) -> Option<&Card> {
        self.stack.current_wrapping()
    }

    /// The cards a swipe stack draws: up to [`crate::stack::STACK_DEPTH`] from the cursor, without wrapping.
    #[must_use]
    pub fn window(&self) -> &[Card] {
        self.stack.window()
    }

    /// The stack's cards in order.
    #[must_use]
    pub fn cards(&self) -> &[Card] {
        self.stack.cards()
    }

    /// The cursor.
    #[must_use]
    pub fn index(&self) -> usize {
        self.stack.index()
    }
}

/// The toast for an undo the deck cards view's Undo already did.
fn already_changed() -> Vec<Effect> {
    vec![Effect::Toast(Toast::quick(
        ToastKind::Info,
        ALREADY_CHANGED,
    ))]
}

/// The toast kind for a deck signal's level.
fn kind_of(level: Level) -> ToastKind {
    match level {
        Level::Info => ToastKind::Info,
        Level::Warning => ToastKind::Warning,
    }
}
