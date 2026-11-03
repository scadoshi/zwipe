//! Remove deck cards: swipe a deck's cards to keep them, remove every copy, or move them between the main board and the maybeboard, with undo that reconciles with the deck's undo log.
//!
//! The view drives zwipe-ui's `screens::remove_cards` model, which owns the deck, the filter, the board, the pile and every write; the view keeps the sheets, the face shown and the card motion.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::time::Instant;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{Board, Deck, DeckCard};
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::remove_cards::{self, Action, Destination, Effect, State};
use zwipe_ui::stack::Direction;
use zwipe_ui::stores::filters::FilterScope;

use super::Screen;
use super::card_sheets::details::DetailsSheet;
use super::card_sheets::filter::FilterSheet;
use super::swipe::UndoFailure;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::screens::card_sheets::printing::PrintingSheet;
use crate::inbound::widgets::card_motion::{CardMotion, Snapshot};
use crate::inbound::widgets::hint;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The Remove screen.
#[derive(Debug, Default)]
pub(crate) struct RemoveView {
    pub(crate) model: State,
    pub(crate) sheet: Option<FilterSheet>,
    pub(crate) details: Option<DetailsSheet>,
    pub(crate) printing: Option<PrintingSheet>,
    pub(crate) face: usize,
    /// The swiped card flying off, or the undone one coming back.
    pub(crate) motion: Option<CardMotion<Snapshot>>,
    /// The side the top card came back from after an undo, which colors its border until the next key.
    pub(crate) entering: Option<Direction>,
}

impl RemoveView {
    /// Whether a card is in motion, so the loop redraws.
    pub(crate) fn animating(&self) -> bool {
        self.motion
            .as_ref()
            .is_some_and(|m| m.running(Instant::now()))
    }

    /// Sends the top card off by `direction` with the face it shows, before the model moves on. Replaces any motion still running.
    pub(crate) fn exit(&mut self, direction: Direction) {
        if let Some(card) = self.model.current().cloned() {
            let snapshot = Snapshot {
                card,
                face: self.face,
            };
            self.motion = Some(CardMotion::exit(direction, snapshot, Instant::now()));
        }
    }

    /// The card under the top one, wrapping around; `None` with fewer than two cards.
    pub(crate) fn next(&self) -> Option<&Card> {
        let cards = self.model.cards();
        if cards.len() < 2 {
            return None;
        }
        let at = self
            .model
            .index()
            .checked_add(1)?
            .checked_rem(cards.len())?;
        cards.get(at)
    }

    /// Opens the filter sheet over the deck's cards.
    pub(crate) fn open_filter(&mut self, cx: &mut Context<'_>) {
        let cards: Vec<Card> = self
            .model
            .entries()
            .iter()
            .map(|e| e.card.clone())
            .collect();
        let config = SheetConfig {
            show_format: true,
            sources: Sources::deck(&cards),
            ..SheetConfig::default()
        };
        self.sheet = Some(FilterSheet::open(cx, self.model.filter(), config));
    }
}

/// Opens Remove for `deck_id` over the current screen under the filter remembered for it, and loads the deck.
pub(crate) fn open(cx: &mut Context<'_>, deck_id: Uuid) {
    let filter = cx
        .shared
        .cards
        .filters
        .restore(FilterScope::Remove, deck_id)
        .unwrap_or_default();
    let mut view = RemoveView::default();
    dispatch(&mut view, Action::Load { deck_id, filter }, cx);
    cx.push(Screen::RemoveCards(Box::new(view)));
}

/// Applies `action` to the model and performs the effects it returns, in order. `TakeUndo` is answered at once: its `Taken` goes to the model after the batch. The face goes back to the front whenever the top card changes.
pub(crate) fn dispatch(view: &mut RemoveView, action: Action, cx: &mut Context<'_>) {
    let mut queue = VecDeque::from([action]);
    while let Some(action) = queue.pop_front() {
        let top = view.model.current().map(|c| c.scryfall_data.id);
        for effect in view.model.update(action) {
            if let Some(next) = perform_effect(view, effect, cx) {
                queue.push_back(next);
            }
        }
        if view.model.current().map(|c| c.scryfall_data.id) != top {
            view.face = 0;
        }
    }
}

/// Performs one effect, returning the action that answers it at once, if any.
fn perform_effect(view: &mut RemoveView, effect: Effect, cx: &mut Context<'_>) -> Option<Action> {
    match effect {
        Effect::OfferHint(offer) => {
            hint::once(cx, offer.key());
        }
        Effect::FetchDeck { ticket, deck_id } => {
            cx.request(Request::FetchDeck { ticket, deck_id }.into());
        }
        Effect::DeleteDeckCard {
            ticket,
            deck_id,
            card_id,
        } => cx.request(
            Request::DeleteDeckCard {
                ticket,
                deck_id,
                card_id,
            }
            .into(),
        ),
        Effect::MoveDeckCard {
            ticket,
            deck_id,
            card_id,
            to,
        } => cx.request(
            Request::MoveDeckCard {
                ticket,
                deck_id,
                card_id,
                to,
            }
            .into(),
        ),
        Effect::TakeUndo {
            ticket,
            deck_id,
            claim,
        } => {
            let taken = cx
                .shared
                .cards
                .undo
                .take_newest(deck_id, |a| claim.matches(a));
            return Some(Action::Taken { ticket, taken });
        }
        Effect::RestoreDeckCard {
            ticket,
            deck_id,
            card,
            board,
            quantity,
        } => cx.request(
            Request::RestoreDeckCard {
                ticket,
                deck_id,
                card,
                board,
                quantity,
            }
            .into(),
        ),
        Effect::RestoreBoard {
            ticket,
            deck_id,
            card_id,
            board,
        } => cx.request(
            Request::RestoreBoard {
                ticket,
                deck_id,
                card_id,
                board,
            }
            .into(),
        ),
        Effect::RecordUndo { deck_id, action } => cx.shared.cards.undo.push(deck_id, action),
        Effect::Enter(side) => {
            view.entering = Some(side);
            view.motion = Some(CardMotion::enter(side, Instant::now()));
        }
        Effect::Navigate(Destination::Previous) => {
            if let Some(deck_id) = view.model.deck_id() {
                cx.shared.cards.filters.park(
                    FilterScope::Remove,
                    deck_id,
                    view.model.filter().clone(),
                );
            }
            cx.back();
        }
        Effect::Toast(toast) => cx.show(toast),
    }
    None
}

/// Remove's work, each of the model's requests carrying its ticket.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    FetchDeck {
        ticket: Ticket,
        deck_id: Uuid,
    },
    /// Deletes every copy.
    DeleteDeckCard {
        ticket: Ticket,
        deck_id: Uuid,
        card_id: Uuid,
    },
    MoveDeckCard {
        ticket: Ticket,
        deck_id: Uuid,
        card_id: Uuid,
        to: Board,
    },
    /// Undoes a remove: creates the card again on its board.
    RestoreDeckCard {
        ticket: Ticket,
        deck_id: Uuid,
        card: Box<Card>,
        board: Board,
        quantity: i32,
    },
    /// Undoes a move: sends the card back to the board it left.
    RestoreBoard {
        ticket: Ticket,
        deck_id: Uuid,
        card_id: Uuid,
        board: Board,
    },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports: the app's names.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::FetchDeck { .. } => "load_deck",
            Self::DeleteDeckCard { .. } => "remove_card",
            Self::MoveDeckCard { .. } => "move_card",
            Self::RestoreDeckCard { .. } => "undo_remove",
            Self::RestoreBoard { .. } => "undo_move",
        }
    }
}

/// Remove's results, each carrying the ticket of the request it answers.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded {
        ticket: Ticket,
        deck: Box<Deck>,
    },
    /// A remove, a move or an undone move went through.
    Landed {
        ticket: Ticket,
    },
    /// An undone remove went through, with the deck card the server made.
    Restored {
        ticket: Ticket,
        deck_card: Box<DeckCard>,
    },
    /// A load, remove or move failed, with its error toast's message.
    Failed {
        ticket: Ticket,
        message: String,
    },
    /// An undo's call failed: a session that could not refresh stops the entrance, a failed call says it could not undo.
    UndoFailed {
        ticket: Ticket,
        failure: UndoFailure,
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

/// The update a request's answer comes back as: `ok`'s on success, else the failure carrying `ticket` and its toast, or the end of the session.
fn reply<T>(
    ticket: Ticket,
    result: Result<T, zwipe_client::ClientError>,
    ok: impl FnOnce(T) -> Update,
) -> screens::Update {
    match result {
        Ok(value) => ok(value).into(),
        Err(e) => screens::failure(&e, |message| Update::Failed { ticket, message }.into()),
    }
}

/// The update an undo's answer comes back as: `ok`'s on success, else the failure carrying `ticket`.
fn undo_reply<T>(
    ticket: Ticket,
    result: Result<T, UndoFailure>,
    ok: impl FnOnce(T) -> Update,
) -> screens::Update {
    match result {
        Ok(value) => ok(value),
        Err(failure) => Update::UndoFailed { ticket, failure },
    }
    .into()
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    Some(match request {
        Request::FetchDeck { ticket, deck_id } => reply(
            ticket,
            async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await,
            |deck| Update::Loaded {
                ticket,
                deck: Box::new(deck),
            },
        ),
        Request::DeleteDeckCard {
            ticket,
            deck_id,
            card_id,
        } => reply(
            ticket,
            async {
                client
                    .delete_deck_card(deck_id, card_id, &account.fresh().await?)
                    .await
            }
            .await,
            |()| Update::Landed { ticket },
        ),
        Request::MoveDeckCard {
            ticket,
            deck_id,
            card_id,
            to,
        } => reply(
            ticket,
            async {
                let request = remove_cards::board_request(to);
                client
                    .update_deck_card(deck_id, card_id, &request, &account.fresh().await?)
                    .await
            }
            .await,
            |_| Update::Landed { ticket },
        ),
        Request::RestoreDeckCard {
            ticket,
            deck_id,
            card,
            board,
            quantity,
        } => {
            let request = remove_cards::restore_request(&card, board, quantity);
            let result = match undo_session(account).await {
                Ok(session) => client
                    .create_deck_card(deck_id, &request, &session)
                    .await
                    .map_err(|e| UndoFailure::Call(e.to_string())),
                Err(failure) => Err(failure),
            };
            undo_reply(ticket, result, |deck_card| Update::Restored {
                ticket,
                deck_card: Box::new(deck_card),
            })
        }
        Request::RestoreBoard {
            ticket,
            deck_id,
            card_id,
            board,
        } => {
            let request = remove_cards::board_request(board);
            let result = match undo_session(account).await {
                Ok(session) => client
                    .update_deck_card(deck_id, card_id, &request, &session)
                    .await
                    .map(drop)
                    .map_err(|e| UndoFailure::Call(e.to_string())),
                Err(failure) => Err(failure),
            };
            undo_reply(ticket, result, |()| Update::Landed { ticket })
        }
    })
}

/// A fresh session for an undo's call, or the failure that stops the undo.
async fn undo_session(
    account: &Account,
) -> Result<zwipe_core::domain::auth::models::session::Session, UndoFailure> {
    account
        .fresh()
        .await
        .map_err(|e| UndoFailure::Session(screens::shown(&e)))
}

/// Hands a reply to the model. A failure raises its toast even when the screen has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::Loaded { ticket, deck } => {
            if let Some(view) = nav.find_mut(pick) {
                view.motion = None;
            }
            Action::Loaded { ticket, deck }
        }
        Update::Landed { ticket } => Action::Landed { ticket },
        Update::Restored { ticket, deck_card } => Action::Restored { ticket, deck_card },
        Update::Failed { ticket, message } => {
            cx.toast(ToastKind::Error, message);
            Action::Failed { ticket }
        }
        Update::UndoFailed { ticket, failure } => {
            let message = match failure {
                UndoFailure::Session(message) => {
                    if let Some(view) = nav.find_mut(pick) {
                        view.entering = None;
                        view.motion = None;
                    }
                    message
                }
                UndoFailure::Call(e) => format!("Failed to undo: {e}"),
            };
            cx.toast(ToastKind::Error, message);
            Action::Failed { ticket }
        }
    };
    if let Some(view) = nav.find_mut(pick) {
        dispatch(view, action, cx);
    }
}
