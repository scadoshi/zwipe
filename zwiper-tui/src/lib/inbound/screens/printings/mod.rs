//! A card's printings: every printing of its oracle card with set, collector number and price, one picked or only browsed.
//!
//! The app's `PrintingSheet`. In [`Mode::Pick`], Enter on another printing leaves it in `cx.shared.caches.handoff.printing`, says "Printing saved" and goes back; the host swaps it into its own list in its arm of `Screen::on_return` (card rows do this with `CardRows::take_printing`). In [`Mode::ReadOnly`] nothing can be picked. Nothing here goes to the server but the fetch of the printings.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::card::Card;

use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The screen's title.
pub(crate) const TITLE: &str = "Printings";

/// The toast after picking a printing, the app's default `saved_message`.
pub(crate) const SAVED: &str = "Printing saved";

/// The toast on leaving with another printing selected but not picked.
pub(crate) const UNCHANGED: &str = "Printing unchanged";

/// Whether a printing can be picked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Enter picks the selected printing and hands it to the screen under.
    Pick,
    /// Browse only.
    ReadOnly,
}

/// The printings of one card, one selected.
#[derive(Debug)]
pub(crate) struct PrintingsView {
    /// The printing the screen opened on.
    pub(crate) card: Card,
    pub(crate) mode: Mode,
    pub(crate) printings: Vec<Card>,
    pub(crate) selected: usize,
    pub(crate) loading: bool,
}

impl PrintingsView {
    /// The selected printing.
    pub(crate) fn selected_printing(&self) -> Option<&Card> {
        self.printings.get(self.selected)
    }

    /// Whether the selection is a printing other than the one the screen opened on.
    pub(crate) fn changed(&self) -> bool {
        self.selected_printing()
            .is_some_and(|p| p.scryfall_data.id != self.card.scryfall_data.id)
    }

    pub(crate) fn next(&mut self) {
        self.selected = self
            .selected
            .saturating_add(1)
            .min(self.printings.len().saturating_sub(1));
    }

    pub(crate) fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// In [`Mode::Pick`], hands the selected printing to the screen under, when it differs from the one opened on, and goes back.
    pub(crate) fn pick(&mut self, cx: &mut Context<'_>) {
        if self.mode != Mode::Pick || !self.changed() {
            return;
        }
        if let Some(printing) = self.selected_printing() {
            cx.shared.caches.handoff.printing = Some(printing.clone());
            cx.toast(ToastKind::Info, SAVED.to_owned());
            cx.back();
        }
    }

    /// Goes back, saying so when a different printing was selected but not picked.
    pub(crate) fn close(&self, cx: &mut Context<'_>) {
        if self.mode == Mode::Pick && self.changed() {
            cx.toast(ToastKind::Info, UNCHANGED.to_owned());
        }
        cx.back();
    }
}

/// The printings' work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// Every printing of the oracle card.
    Fetch { oracle_id: Uuid },
}

/// The printings' results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded {
        oracle_id: Uuid,
        printings: Vec<Card>,
    },
    Failed {
        oracle_id: Uuid,
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

/// Opens the printings of `card`, selected on `card` itself once they load. A card with no oracle id has only itself to show.
pub(crate) fn open(cx: &mut Context<'_>, card: Card, mode: Mode) {
    let oracle_id = card.scryfall_data.oracle_id;
    let mut view = PrintingsView {
        printings: Vec::new(),
        selected: 0,
        loading: oracle_id.is_some(),
        mode,
        card,
    };
    match oracle_id {
        Some(oracle_id) => cx.request(Request::Fetch { oracle_id }.into()),
        None => view.printings.push(view.card.clone()),
    }
    cx.push(Screen::Printings(Box::new(view)));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request::Fetch { oracle_id } = request;
    // Printings are public, so no session is needed and none can end.
    let update = match account.client.fetch_printings(oracle_id).await {
        Ok(printings) => Update::Loaded {
            oracle_id,
            printings,
        },
        Err(e) => Update::Failed {
            oracle_id,
            message: crate::inbound::screens::shown(&e),
        },
    };
    Some(update.into())
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let oracle_id = match &update {
        Update::Loaded { oracle_id, .. } | Update::Failed { oracle_id, .. } => *oracle_id,
    };
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    if view.card.scryfall_data.oracle_id != Some(oracle_id) {
        return;
    }
    view.loading = false;
    match update {
        Update::Loaded { printings, .. } => {
            let current = view.card.scryfall_data.id;
            view.selected = printings
                .iter()
                .position(|p| p.scryfall_data.id == current)
                .unwrap_or(0);
            view.printings = printings;
        }
        Update::Failed { message, .. } => cx.toast(ToastKind::Error, message),
    }
}
