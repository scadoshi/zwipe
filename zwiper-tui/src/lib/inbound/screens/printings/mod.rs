//! A card's printings: every printing of its oracle card with set, collector number and price, one picked or only browsed.
//!
//! The app's `PrintingSheet`. What the sheet knows and decides (what loads, which printing it opens on, whether Save may fire, what each way out toasts) is zwipe-ui's [`State`]; this view adds the card it opened on and the cursor. In [`Mode::Pick`], Enter on another printing leaves it in `cx.shared.caches.handoff.printing`, says "Printing saved" and goes back; the host swaps it into its own list in its arm of `Screen::on_return` (card rows do this with `CardRows::take_printing`). In [`Mode::ReadOnly`] nothing can be picked. Nothing here goes to the server but the fetch of the printings.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_ui::generation::Ticket;
pub(crate) use zwipe_ui::screens::printing_sheet::Mode;
use zwipe_ui::screens::printing_sheet::{Action, Effect, SAVED, State};

use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The printings of one card: the model plus the card it opened on and the cursor.
#[derive(Debug)]
pub(crate) struct PrintingsView {
    pub(crate) model: State,
    /// The printing the screen opened on, whose name heads it and whose row reads `current`.
    pub(crate) card: Card,
    /// The printing under the cursor, as an index into the model's list.
    pub(crate) selected: usize,
}

impl PrintingsView {
    /// The printing under the cursor, or `None` while the list is empty.
    pub(crate) fn selected_printing(&self) -> Option<&Card> {
        self.model.visible(self.selected)
    }

    pub(crate) fn next(&mut self) {
        self.selected = self
            .selected
            .saturating_add(1)
            .min(self.model.printings().len().saturating_sub(1));
    }

    pub(crate) fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}

/// Applies `action` to the model and performs the effects it returns, in order.
pub(crate) fn dispatch(view: &mut PrintingsView, action: Action, cx: &mut Context<'_>) {
    for effect in view.model.update(action) {
        match effect {
            Effect::FetchPrintings { ticket, oracle_id } => {
                cx.request(Request::Fetch { ticket, oracle_id }.into());
            }
            Effect::Show(at) | Effect::Rewind(at) => view.selected = at,
            Effect::Pick(printing) => cx.shared.caches.handoff.printing = Some(*printing),
            Effect::Toast(toast) => cx.show(toast),
            Effect::Close => cx.back(),
        }
    }
}

/// The printings' work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// Every printing of the oracle card.
    Fetch { ticket: Ticket, oracle_id: Uuid },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Fetch { .. } => "fetch",
        }
    }
}

/// The printings' results, each carrying its request's ticket back to the model.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded {
        ticket: Ticket,
        printings: Vec<Card>,
    },
    Failed {
        ticket: Ticket,
        message: String,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Printings(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Printings(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut PrintingsView> {
    match screen {
        Screen::Printings(view) => Some(view),
        _ => None,
    }
}

/// Opens the printings of `card`, on `card` itself once they load. A card with no oracle id has nothing to fetch by and lists nothing, as in the app.
pub(crate) fn open(cx: &mut Context<'_>, card: Card, mode: Mode) {
    let mut view = PrintingsView {
        model: State::new(mode, SAVED),
        selected: 0,
        card,
    };
    let action = Action::Open {
        printing_id: view.card.scryfall_data.id,
        oracle_id: view.card.scryfall_data.oracle_id,
    };
    dispatch(&mut view, action, cx);
    cx.push(Screen::Printings(Box::new(view)));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request::Fetch { ticket, oracle_id } = request;
    // Printings are public, so no session is needed and none can end.
    let update = match account.client.fetch_printings(oracle_id).await {
        Ok(printings) => Update::Loaded { ticket, printings },
        Err(e) => Update::Failed {
            ticket,
            message: screens::shown(&e),
        },
    };
    Some(update.into())
}

/// Hands the printings to the model. A failure raises its toast even when the screen has closed.
pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let action = match update {
        Update::Loaded { ticket, printings } => Action::Loaded { ticket, printings },
        Update::Failed { ticket, message } => {
            cx.toast(ToastKind::Error, message);
            Action::Failed { ticket }
        }
    };
    if let Some(view) = nav.find_mut(pick) {
        dispatch(view, action, cx);
    }
}
