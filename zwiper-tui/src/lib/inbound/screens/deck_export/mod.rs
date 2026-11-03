//! Export: the deck as a decklist that Import reads back, with toggles for the boards it includes and a Copy action.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]; this view adds the focused board chip and how far the list is scrolled.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::deck::Deck;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::deck_export::{Action, Destination, Effect, State};

use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The Export screen's state: the model plus the focused board chip and how far the list is scrolled.
#[derive(Debug)]
pub(crate) struct ExportView {
    pub(crate) model: State,
    /// The focused chip, an index into [`zwipe_ui::screens::deck_export::Board::ALL`].
    pub(crate) focus: usize,
    pub(crate) scroll: u16,
}

impl Default for ExportView {
    fn default() -> Self {
        Self {
            model: State::new(),
            focus: 0,
            scroll: 0,
        }
    }
}

/// Shows the export hint again.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, OneTimeHint::Export.key());
}

/// Applies `action` to the model and performs the effects it returns, in order.
pub(crate) fn dispatch(view: &mut ExportView, action: Action, cx: &mut Context<'_>) {
    for effect in view.model.update(action) {
        match effect {
            Effect::OfferHint(offer) => {
                hint::once(cx, offer.key());
            }
            Effect::FetchDeck { ticket, deck_id } => {
                cx.request(Request::Load { ticket, deck_id }.into());
            }
            Effect::Copy { text, done } => screens::deck::copy(cx, &text, done),
            Effect::Navigate(Destination::Deck(_)) => cx.back(),
        }
    }
}

/// The Export screen's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    Load { ticket: Ticket, deck_id: Uuid },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Load { .. } => "load",
        }
    }
}

/// The Export screen's results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded { ticket: Ticket, deck: Box<Deck> },
    Failed { ticket: Ticket, error: String },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::DeckExport(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::DeckExport(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut ExportView> {
    match screen {
        Screen::DeckExport(view) => Some(view),
        _ => None,
    }
}

/// Opens Export for `deck_id`, fetches the deck, and shows the hint once.
pub(crate) fn open(cx: &mut Context<'_>, deck_id: Uuid) {
    let mut view = ExportView::default();
    dispatch(&mut view, Action::Load(deck_id), cx);
    cx.push(Screen::DeckExport(Box::new(view)));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let Request::Load { ticket, deck_id } = request;
    Some(
        match async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await {
            Ok(deck) => Update::Loaded {
                ticket,
                deck: Box::new(deck),
            }
            .into(),
            Err(e) => screens::failure(&e, |error| Update::Failed { ticket, error }.into()),
        },
    )
}

/// Hands a reply to the model. A failure raises its toast even when the screen has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::Loaded { ticket, deck } => Action::Loaded { ticket, deck },
        Update::Failed { ticket, error } => {
            cx.toast(ToastKind::Error, error);
            Action::Failed { ticket }
        }
    };
    if let Some(view) = nav.find_mut(pick) {
        dispatch(view, action, cx);
    }
}
