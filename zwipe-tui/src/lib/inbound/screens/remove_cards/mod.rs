//! Remove deck cards: swipe a deck's cards to keep them, remove every copy, or move them between the main board and the maybeboard, with undo that reconciles with the deck's undo log.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::deck_metrics::deck_price;
use zwipe_core::domain::deck::{Board, Deck, DeckCard, DeckEntry, DeckProfile};
use zwipe_core::domain::user::hints::HINT_REMOVE_DECK_CARDS;
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};

use super::Screen;
use super::card_sheets::details::DetailsSheet;
use super::card_sheets::filter::{FilterSheet, SheetConfig};
use super::swipe::{UndoFailure, signal};
use crate::domain::deck_signals::{self, Crossing};
use crate::domain::filter::picker::Sources;
use crate::domain::stack::{CardStack, RemoveAction, StackAction};
use crate::domain::stores::filters::FilterScope;
use crate::domain::stores::undo::{UndoAction, create_board_field};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::screens::card_sheets::printing::PrintingSheet;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// Which board is being trimmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum BoardFilter {
    #[default]
    Main,
    Maybe,
    Side,
    All,
}

impl BoardFilter {
    pub(crate) const ALL: [Self; 4] = [Self::Main, Self::Maybe, Self::Side, Self::All];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Main => "Main",
            Self::Maybe => "Maybe",
            Self::Side => "Side",
            Self::All => "All",
        }
    }

    fn keeps(self, board: Board) -> bool {
        match self {
            Self::Main => board.is_active(),
            Self::Maybe => board.is_maybeboard(),
            Self::Side => board.is_sideboard(),
            Self::All => true,
        }
    }
}

/// The Remove screen.
#[derive(Debug)]
pub(crate) struct RemoveView {
    pub(crate) deck_id: Uuid,
    pub(crate) loaded: bool,
    pub(crate) profile: Option<DeckProfile>,
    pub(crate) entries: Vec<DeckEntry>,
    pub(crate) command_zone_cards: Vec<Card>,
    pub(crate) filter: CardQueryBuilder,
    pub(crate) board: BoardFilter,
    pub(crate) stack: CardStack<RemoveAction>,
    pub(crate) sheet: Option<FilterSheet>,
    pub(crate) details: Option<DetailsSheet>,
    pub(crate) printing: Option<PrintingSheet>,
    pub(crate) face: usize,
}

impl RemoveView {
    fn currency(&self) -> PriceCurrency {
        self.profile
            .as_ref()
            .and_then(|p| p.price_target_currency)
            .unwrap_or(PriceCurrency::Usd)
    }

    fn price(&self) -> f64 {
        deck_price(&self.entries, &self.command_zone_cards, self.currency())
    }

    /// The board, the filter and the sort, into a fresh stack; history stays.
    pub(crate) fn rerun(&mut self) {
        let board = self.board;
        let cards: Vec<Card> = self
            .entries
            .iter()
            .filter(|e| board.keeps(e.deck_card.board))
            .map(|e| e.card.clone())
            .collect();
        let mut cards = Cards::from(cards);
        if !self.filter.is_empty()
            && let Ok(criteria) = self.filter.build_criteria()
        {
            cards = cards.matching(&criteria);
        }
        let cards = cards.sorted(self.filter.sort(), self.filter.direction());
        self.stack.replace(cards.into());
        self.face = 0;
    }

    pub(crate) fn current(&self) -> Option<&Card> {
        self.stack.current_wrapping()
    }

    fn entry(&self, card_id: Uuid) -> Option<&DeckEntry> {
        self.entries
            .iter()
            .find(|e| e.deck_card.scryfall_data_id == card_id)
    }

    /// Left: keep it and cycle on.
    pub(crate) fn keep(&mut self, cx: &mut Context<'_>) {
        if self.current().is_none() {
            return;
        }
        self.stack.record(RemoveAction::Keep);
        cx.toast_for(ToastKind::Info, "Kept".to_owned(), toast::QUICK);
        self.stack.advance_wrapping();
        self.face = 0;
    }

    /// Right: remove every copy.
    pub(crate) fn remove(&mut self, cx: &mut Context<'_>) {
        let Some(card) = self.current().cloned() else {
            return;
        };
        let id = card.scryfall_data.id;
        let entry = self.entry(id).cloned();
        let (board, quantity) = entry.as_ref().map_or((Board::Deck, 1), |e| {
            (e.deck_card.board, *e.deck_card.quantity)
        });
        self.stack.record(RemoveAction::Remove {
            card: Box::new(card),
            board,
            quantity,
        });
        if let Some(entry) = entry {
            cx.request(
                Request::Remove {
                    deck_id: self.deck_id,
                    entry: Box::new(entry),
                }
                .into(),
            );
        }
        cx.toast_for(
            ToastKind::Success,
            "Removed from deck".to_owned(),
            toast::QUICK,
        );
        let (lands, price) = (deck_signals::land_count(&self.entries), self.price());
        self.entries.retain(|e| e.deck_card.scryfall_data_id != id);
        self.stack.remove_current();
        self.signals(cx, lands, price);
        self.face = 0;
    }

    /// Up: between the main board and the maybeboard; the sideboard goes to the maybeboard.
    pub(crate) fn move_board(&mut self, cx: &mut Context<'_>) {
        let Some(card) = self.current().cloned() else {
            return;
        };
        let id = card.scryfall_data.id;
        let from = self.entry(id).map_or(Board::Deck, |e| e.deck_card.board);
        let to = if from.is_maybeboard() {
            Board::Deck
        } else {
            Board::Maybeboard
        };
        self.stack.record(RemoveAction::MoveBoard {
            card: Box::new(card.clone()),
            from,
            to,
        });
        cx.request(
            Request::Move {
                deck_id: self.deck_id,
                card: Box::new(card),
                from,
                to,
            }
            .into(),
        );
        let message = if to == Board::Deck {
            "Moved to main"
        } else {
            "Moved to maybeboard"
        };
        cx.toast_for(ToastKind::Info, message.to_owned(), toast::QUICK);
        let (lands, price) = (deck_signals::land_count(&self.entries), self.price());
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.deck_card.scryfall_data_id == id)
        {
            entry.deck_card.board = to;
        }
        self.stack.remove_current();
        self.signals(cx, lands, price);
        self.face = 0;
    }

    fn signals(&self, cx: &mut Context<'_>, lands: i32, price: f64) {
        let profile = self.profile.as_ref();
        signal(
            cx,
            deck_signals::land_crossing(
                profile.and_then(|p| p.land_target),
                lands,
                deck_signals::land_count(&self.entries),
                Crossing::Down,
            ),
        );
        signal(
            cx,
            deck_signals::budget_crossing(
                profile.and_then(|p| p.price_target),
                self.currency(),
                price,
                self.price(),
            ),
        );
    }

    /// Down: undoes the last swipe, skipping the server when the deck cards Undo already did it.
    pub(crate) fn undo(&mut self, cx: &mut Context<'_>) {
        let Some(action) = self.stack.pop_action() else {
            cx.toast(ToastKind::Info, "Nothing to undo".to_owned());
            return;
        };
        self.stack.prime_entering(action.exited());
        self.face = 0;
        let deck_id = self.deck_id;
        match action {
            RemoveAction::Keep => {
                if !self.stack.retreat_wrapping() {
                    self.stack.cancel_entering();
                }
                cx.toast_for(ToastKind::Info, "Undid keep".to_owned(), toast::QUICK);
            }
            RemoveAction::Remove {
                card,
                board,
                quantity,
            } => {
                let id = card.scryfall_data.id;
                let taken = cx.shared.cards.undo.take_newest(deck_id, |a| {
                    matches!(a, UndoAction::Removed { entry, .. } if entry.deck_card.scryfall_data_id == id)
                });
                if taken.is_none() && self.entry(id).is_some() {
                    self.stack.cancel_entering();
                    cx.toast_for(ToastKind::Info, "Already changed".to_owned(), toast::QUICK);
                    return;
                }
                self.stack.insert_current((*card).clone());
                cx.request(
                    Request::UndoRemove {
                        deck_id,
                        card,
                        board,
                        quantity,
                        taken: taken.map(Box::new),
                    }
                    .into(),
                );
            }
            RemoveAction::MoveBoard { card, from, .. } => {
                let id = card.scryfall_data.id;
                let taken = cx.shared.cards.undo.take_newest(deck_id, |a| {
                    matches!(a, UndoAction::MovedBoard { card_id, from: f, .. } if *card_id == id && *f == from)
                });
                let already_back = self.entry(id).is_some_and(|e| e.deck_card.board == from);
                if taken.is_none() && already_back {
                    self.stack.cancel_entering();
                    cx.toast_for(ToastKind::Info, "Already changed".to_owned(), toast::QUICK);
                    return;
                }
                self.stack.insert_current((*card).clone());
                cx.request(
                    Request::UndoMove {
                        deck_id,
                        card,
                        from,
                        taken: taken.map(Box::new),
                    }
                    .into(),
                );
            }
        }
    }

    /// Refresh: back to the first card with no history; a Random sort reshuffles.
    pub(crate) fn refresh(&mut self, cx: &mut Context<'_>) {
        self.stack.rewind();
        if self.filter.sort() == Some(CardSortKey::Random) {
            self.rerun();
        }
        self.face = 0;
        cx.toast_for(ToastKind::Info, "Stack refreshed".to_owned(), toast::QUICK);
    }

    pub(crate) fn set_board(&mut self, board: BoardFilter) {
        self.board = board;
        self.rerun();
    }

    pub(crate) fn open_filter(&mut self, cx: &mut Context<'_>) {
        let cards: Vec<Card> = self.entries.iter().map(|e| e.card.clone()).collect();
        let config = SheetConfig {
            show_format: true,
            sources: Sources::deck(&cards),
            ..SheetConfig::default()
        };
        self.sheet = Some(FilterSheet::open(cx, &self.filter, config));
    }

    /// Leaves, parking the filter.
    pub(crate) fn leave(&mut self, cx: &mut Context<'_>) {
        cx.shared
            .cards
            .filters
            .park(FilterScope::Remove, self.deck_id, self.filter.clone());
        cx.back();
    }
}

/// Opens Remove for `deck_id` over the current screen and loads the deck.
pub(crate) fn open(cx: &mut Context<'_>, deck_id: Uuid) {
    let filter = cx
        .shared
        .cards
        .filters
        .restore(FilterScope::Remove, deck_id)
        .unwrap_or_default();
    let view = RemoveView {
        deck_id,
        loaded: false,
        profile: None,
        entries: Vec::new(),
        command_zone_cards: Vec::new(),
        filter,
        board: BoardFilter::Main,
        stack: CardStack::default(),
        sheet: None,
        details: None,
        printing: None,
        face: 0,
    };
    hint::once(cx, HINT_REMOVE_DECK_CARDS);
    cx.request(Request::Load { deck_id }.into());
    cx.push(Screen::RemoveCards(Box::new(view)));
}

/// Remove's work.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    Load {
        deck_id: Uuid,
    },
    /// Deletes every copy.
    Remove {
        deck_id: Uuid,
        entry: Box<DeckEntry>,
    },
    Move {
        deck_id: Uuid,
        card: Box<Card>,
        from: Board,
        to: Board,
    },
    UndoRemove {
        deck_id: Uuid,
        card: Box<Card>,
        board: Board,
        quantity: i32,
        taken: Option<Box<UndoAction>>,
    },
    UndoMove {
        deck_id: Uuid,
        card: Box<Card>,
        from: Board,
        taken: Option<Box<UndoAction>>,
    },
}

/// Remove's results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded {
        deck_id: Uuid,
        deck: Box<Deck>,
    },
    /// A remove landed, which the undo log records.
    Removed {
        deck_id: Uuid,
        entry: Box<DeckEntry>,
    },
    /// A move landed, which the undo log records.
    Moved {
        deck_id: Uuid,
        card: Box<Card>,
        from: Board,
    },
    UndidRemove {
        deck_id: Uuid,
        card: Box<Card>,
        taken: Option<Box<UndoAction>>,
        result: Result<Box<DeckCard>, UndoFailure>,
    },
    UndidMove {
        deck_id: Uuid,
        card: Box<Card>,
        from: Board,
        taken: Option<Box<UndoAction>>,
        result: Result<(), UndoFailure>,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::RemoveCards(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::RemoveCards(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut RemoveView> {
    match screen {
        Screen::RemoveCards(view) => Some(view),
        _ => None,
    }
}

fn toast(message: String) -> screens::Update {
    screens::Update::Toast(ToastKind::Error, message)
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let session = account.fresh().await;
    let update = match request {
        Request::Load { deck_id } => {
            let session = match session {
                Ok(s) => s,
                Err(e) => {
                    return Some(screens::failure(&e, |m| {
                        screens::Update::Toast(ToastKind::Error, m)
                    }));
                }
            };
            match client.fetch_deck(deck_id, &session).await {
                Ok(deck) => Update::Loaded {
                    deck_id,
                    deck: Box::new(deck),
                },
                Err(e) => return Some(toast(e.to_user_message())),
            }
        }
        Request::Remove { deck_id, entry } => {
            let result = match session {
                Ok(s) => {
                    client
                        .delete_deck_card(deck_id, entry.deck_card.scryfall_data_id, &s)
                        .await
                }
                Err(e) => Err(e),
            };
            match result {
                Ok(()) => Update::Removed { deck_id, entry },
                Err(e) => return Some(toast(e.to_user_message())),
            }
        }
        Request::Move {
            deck_id,
            card,
            from,
            to,
        } => {
            let patch = HttpPatchDeckCard::new(None, Some(to.display_name().to_owned()));
            let result = match session {
                Ok(s) => client
                    .update_deck_card(deck_id, card.scryfall_data.id, &patch, &s)
                    .await
                    .map(drop),
                Err(e) => Err(e),
            };
            match result {
                Ok(()) => Update::Moved {
                    deck_id,
                    card,
                    from,
                },
                Err(e) => return Some(toast(e.to_user_message())),
            }
        }
        Request::UndoRemove {
            deck_id,
            card,
            board,
            quantity,
            taken,
        } => {
            let result = undo_remove(client, session, deck_id, &card, board, quantity).await;
            Update::UndidRemove {
                deck_id,
                card,
                taken,
                result,
            }
        }
        Request::UndoMove {
            deck_id,
            card,
            from,
            taken,
        } => {
            let result = undo_move(client, session, deck_id, &card, from).await;
            Update::UndidMove {
                deck_id,
                card,
                from,
                taken,
                result,
            }
        }
    };
    Some(update.into())
}

/// Adds `quantity` copies of `card` back to `board`, undoing a remove.
async fn undo_remove(
    client: &ZwipeClient,
    session: Result<Session, ClientError>,
    deck_id: Uuid,
    card: &Card,
    board: Board,
    quantity: i32,
) -> Result<Box<DeckCard>, UndoFailure> {
    let s = session.map_err(|e| UndoFailure::Session(e.to_user_message()))?;
    let request = HttpCreateDeckCard::new(&card.scryfall_data, quantity, create_board_field(board));
    client
        .create_deck_card(deck_id, &request, &s)
        .await
        .map(Box::new)
        .map_err(|e| UndoFailure::Call(e.to_string()))
}

/// Moves `card` back to the board it came `from`, undoing a move.
async fn undo_move(
    client: &ZwipeClient,
    session: Result<Session, ClientError>,
    deck_id: Uuid,
    card: &Card,
    from: Board,
) -> Result<(), UndoFailure> {
    let s = session.map_err(|e| UndoFailure::Session(e.to_user_message()))?;
    let patch = HttpPatchDeckCard::new(None, Some(from.display_name().to_owned()));
    client
        .update_deck_card(deck_id, card.scryfall_data.id, &patch, &s)
        .await
        .map(drop)
        .map_err(|e| UndoFailure::Call(e.to_string()))
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    // Landed swipes record in the undo log, and failed undos give their entry back, whether or not the screen is open.
    match &update {
        Update::Removed { deck_id, entry } => {
            let baseline = *entry.deck_card.quantity;
            cx.shared.cards.undo.push(
                *deck_id,
                UndoAction::Removed {
                    entry: entry.clone(),
                    baseline,
                },
            );
            return;
        }
        Update::Moved {
            deck_id,
            card,
            from,
        } => {
            cx.shared.cards.undo.push(
                *deck_id,
                UndoAction::MovedBoard {
                    card_id: card.scryfall_data.id,
                    card_name: card.scryfall_data.name.clone(),
                    from: *from,
                },
            );
            return;
        }
        Update::UndidRemove {
            deck_id,
            taken,
            result: Err(failure),
            ..
        }
        | Update::UndidMove {
            deck_id,
            taken,
            result: Err(failure),
            ..
        } => {
            let message = match failure {
                UndoFailure::Session(m) => m.clone(),
                UndoFailure::Call(e) => format!("Failed to undo: {e}"),
            };
            cx.toast(ToastKind::Error, message);
            if let Some(taken) = taken {
                cx.shared.cards.undo.push(*deck_id, (**taken).clone());
            }
        }
        _ => {}
    }
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    match update {
        Update::Loaded { deck_id, deck } => {
            if deck_id != view.deck_id {
                return;
            }
            let deck = *deck;
            view.entries = deck.entries;
            view.command_zone_cards = deck.command_zone_cards;
            view.profile = Some(deck.deck_profile);
            view.loaded = true;
            view.rerun();
        }
        Update::Removed { .. } | Update::Moved { .. } => {}
        Update::UndidRemove {
            deck_id,
            card,
            result,
            ..
        } => {
            if deck_id != view.deck_id {
                return;
            }
            undid_remove(view, cx, *card, result);
        }
        Update::UndidMove {
            deck_id,
            card,
            from,
            result,
            ..
        } => {
            if deck_id != view.deck_id {
                return;
            }
            undid_move(view, cx, &card, from, &result);
        }
    }
}

/// Settles an undone remove on the open screen: the card returns, or the stack steps back.
fn undid_remove(
    view: &mut RemoveView,
    cx: &mut Context<'_>,
    card: Card,
    result: Result<Box<DeckCard>, UndoFailure>,
) {
    match result {
        Ok(deck_card) => {
            view.entries.push(DeckEntry {
                card,
                deck_card: *deck_card,
            });
            cx.toast_for(ToastKind::Success, "Undid remove".to_owned(), toast::QUICK);
        }
        Err(UndoFailure::Session(_)) => view.stack.cancel_entering(),
        Err(UndoFailure::Call(_)) => {}
    }
}

/// Settles an undone move on the open screen: the card goes back to `from`, or the stack steps back.
fn undid_move(
    view: &mut RemoveView,
    cx: &mut Context<'_>,
    card: &Card,
    from: Board,
    result: &Result<(), UndoFailure>,
) {
    match result {
        Ok(()) => {
            let id = card.scryfall_data.id;
            if let Some(entry) = view
                .entries
                .iter_mut()
                .find(|e| e.deck_card.scryfall_data_id == id)
            {
                entry.deck_card.board = from;
            }
            cx.toast_for(ToastKind::Success, "Undid move".to_owned(), toast::QUICK);
        }
        Err(UndoFailure::Session(_)) => view.stack.cancel_entering(),
        Err(UndoFailure::Call(_)) => {}
    }
}
