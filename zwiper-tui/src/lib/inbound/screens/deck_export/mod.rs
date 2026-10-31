//! Export: the deck as a decklist that Import reads back, with toggles for the boards it includes and a Copy action.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use uuid::Uuid;
use zwipe_core::domain::deck::Deck;

use super::{Screen, failure};
use crate::domain::deck::export::{self, Boards};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The key the server stores the export hint under.
pub(crate) const HINT_KEY: &str = "export";

/// The Export screen: the deck once loaded, the boards it includes, the focused board chip, and how far the list is scrolled.
#[derive(Debug)]
pub(crate) struct ExportView {
    pub(crate) deck_id: Uuid,
    pub(crate) deck: Option<Deck>,
    pub(crate) failed: bool,
    pub(crate) boards: Boards,
    /// The focused chip: 0 Deck, 1 Maybe, 2 Side.
    pub(crate) focus: usize,
    pub(crate) scroll: u16,
}

impl ExportView {
    pub(crate) fn opening(deck_id: Uuid) -> Self {
        Self {
            deck_id,
            deck: None,
            failed: false,
            boards: Boards::default(),
            focus: 0,
            scroll: 0,
        }
    }

    /// The decklist, once the deck has loaded.
    pub(crate) fn text(&self) -> Option<String> {
        self.deck.as_ref().map(|d| export::text(d, self.boards))
    }

    /// Toggles the focused board chip under the at-least-one rule.
    pub(crate) fn toggle(&mut self) {
        match self.focus {
            0 => self.boards.toggle_deck(),
            1 => self.boards.toggle_maybe(),
            _ => self.boards.toggle_side(),
        }
    }

    /// Copies the decklist, or does nothing before it loads.
    pub(crate) fn copy(&self, cx: &mut Context<'_>) {
        if let Some(text) = self.text() {
            screens::deck::copy(cx, &text, "Copied to clipboard");
        }
    }
}

/// Shows the export hint again.
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_KEY);
}

/// The Export screen's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    Load(Uuid),
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Load(..) => "load",
        }
    }
}

/// The Export screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded(Box<Deck>),
    Failed(String),
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
    cx.push(Screen::DeckExport(Box::new(ExportView::opening(deck_id))));
    cx.request(Request::Load(deck_id).into());
    hint::once(cx, HINT_KEY);
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    match request {
        Request::Load(deck_id) => Some(
            match async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(deck) => Update::Loaded(Box::new(deck)).into(),
                Err(e) => failure(&e, |m| Update::Failed(m).into()),
            },
        ),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    match update {
        Update::Loaded(deck) => {
            if deck.deck_profile.id == view.deck_id {
                view.deck = Some(*deck);
            }
        }
        Update::Failed(message) => {
            view.failed = true;
            cx.toast(ToastKind::Error, message);
        }
    }
}
