//! The deck list: the user's decks, sorted by name, one selected.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_core::domain::deck::DeckProfile;

use super::{Screen, failure};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::outbound::account::Account;

/// The user's decks.
#[derive(Debug, Default)]
pub(crate) struct DeckList {
    pub(crate) decks: Vec<DeckProfile>,
    pub(crate) selected: usize,
    pub(crate) loading: bool,
    pub(crate) error: Option<String>,
}

impl DeckList {
    pub(crate) fn loading() -> Self {
        Self {
            loading: true,
            ..Self::default()
        }
    }

    /// Takes the decks, sorted by name ignoring case as in the app, keeping the selection in range.
    pub(crate) fn set_decks(&mut self, mut decks: Vec<DeckProfile>) {
        decks.sort_by_cached_key(|d| d.name.to_lowercase());
        self.decks = decks;
        self.loading = false;
        self.selected = self.selected.min(self.decks.len().saturating_sub(1));
    }

    pub(crate) fn selected_deck(&self) -> Option<&DeckProfile> {
        self.decks.get(self.selected)
    }

    pub(crate) fn next(&mut self) {
        self.selected = self
            .selected
            .saturating_add(1)
            .min(self.decks.len().saturating_sub(1));
    }

    pub(crate) fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Fetches the decks again, keeping the selection.
    pub(crate) fn reload(&mut self, cx: &mut Context<'_>) {
        cx.shared.status.clear();
        self.loading = true;
        self.error = None;
        cx.request(Request::Load.into());
    }
}

/// The deck list's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    Load,
}

/// The deck list's results.
#[derive(Debug)]
pub(crate) enum Update {
    Loaded(Vec<DeckProfile>),
    Failed(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Decks(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Decks(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut DeckList> {
    match screen {
        Screen::Decks(list) => Some(list),
        _ => None,
    }
}

/// Opens the deck list over the current screen and fetches the decks.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.shared.status.clear();
    cx.push(Screen::Decks(DeckList::loading()));
    cx.request(Request::Load.into());
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    match request {
        Request::Load => Some(
            match async { client.fetch_deck_profiles(&account.fresh().await?).await }.await {
                Ok(decks) => Update::Loaded(decks).into(),
                Err(e) => failure(&e, |m| Update::Failed(m).into()),
            },
        ),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, _cx: &mut Context<'_>) {
    let Some(list) = nav.find_mut(pick) else {
        return;
    };
    match update {
        Update::Loaded(decks) => list.set_decks(decks),
        Update::Failed(error) => {
            list.loading = false;
            list.error = Some(error);
        }
    }
}
