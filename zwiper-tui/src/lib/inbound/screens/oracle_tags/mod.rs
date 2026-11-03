//! The oracle tag dictionary, and the app-wide oracle tag catalog it reads.
//!
//! # The catalog
//!
//! `shared.caches.oracle_tags` holds the catalog for the whole launch, as the app's `CatalogCache` does. [`ensure_catalog`] fetches it with `fetch_oracle_tags` when it is missing, failed or a day old; it is single flight, and a refetch keeps the old tags on screen until the new ones land (stale while revalidate). Its result is written to the cache whether or not the dictionary is open. Every screen that shows tag descriptions calls [`ensure_catalog`] when it opens.
//!
//! # The dictionary
//!
//! A search box, a letter rail A to Z (plus `#` when some slug starts with something else), and the tags under the letter or matching the search, each with its description and parents. Each tag offers Examples (the cards that carry it) and Use. What the dictionary knows and decides (the search, the letter, the rows and the lines in place of them, where Examples and Use lead) is zwipe-ui's [`State`]; this view adds the search box, typing and the selected row.
//!
//! # Use, and what a host does
//!
//! The dictionary opens over the screen that wants a tag (the host) and never knows who that is. Use leaves the slug in `cx.shared.caches.handoff.oracle_tag` and goes back. The host adopts it in its arm of `Screen::on_return` by taking it:
//!
//! ```text
//! if let Some(slug) = cx.shared.caches.handoff.oracle_tag.take() {
//!     // adopt slug
//! }
//! ```
//!
//! Taking it matters: a value left behind would be adopted again on the next return. [`open`] clears any stale value before the dictionary shows. Going back without Use leaves the handoff empty.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::Instant;

use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::user::models::hints::HINT_ORACLE_TAG_DICTIONARY;
use zwipe_ui::screens::oracle_tag_dictionary::{self as model, Action, Destination, Effect, State};

use super::Screen;
use super::home::cache::Catalog;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::screens::otag_examples;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::text_input::TextInput;
use crate::outbound::account::Account;

/// The dictionary's state: the model, which holds the search and the letter, plus what only the terminal keeps.
#[derive(Debug)]
pub(crate) struct OracleTagsView {
    pub(crate) model: State,
    /// The box the search is typed in. Its value follows each keystroke into the model.
    pub(crate) search: TextInput,
    /// Keys go to the search box rather than the list.
    pub(crate) typing: bool,
    /// The selected row in the list on screen.
    pub(crate) selected: usize,
}

impl Default for OracleTagsView {
    /// Browsing A with an empty search.
    fn default() -> Self {
        Self {
            model: State::new(),
            search: TextInput::default(),
            typing: false,
            selected: 0,
        }
    }
}

impl OracleTagsView {
    /// The tags on screen, empty when the model draws a line in place of them.
    pub(crate) fn rows(&self, tags: &[OracleTag]) -> Vec<OracleTag> {
        match self.model.list(tags) {
            model::List::Rows(rows) => rows,
            model::List::Empty(_) => Vec::new(),
        }
    }

    /// Browses the letter `step` places along `rail` from the one browsed, stopping at either end. The model clears the search, and the box follows.
    pub(crate) fn step_letter(&mut self, rail: &[char], step: Step, cx: &mut Context<'_>) {
        let at = rail
            .iter()
            .position(|&letter| letter == self.model.letter())
            .unwrap_or(0);
        let to = match step {
            Step::Back => at.saturating_sub(1),
            Step::Forward => at.saturating_add(1).min(rail.len().saturating_sub(1)),
        };
        let letter = rail.get(to).copied().unwrap_or('a');
        self.search.set_value("");
        self.selected = 0;
        dispatch(self, Action::PickLetter(letter), cx);
    }

    /// Moves the selection one row, staying within `count` rows.
    pub(crate) fn step_row(&mut self, count: usize, step: Step) {
        self.selected = match step {
            Step::Back => self.selected.saturating_sub(1),
            Step::Forward => self.selected.saturating_add(1).min(count.saturating_sub(1)),
        };
    }
}

/// A move along the rail or the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Back,
    Forward,
}

/// What the app-wide catalog holds, as the model reads it.
pub(crate) fn catalog(cache: &Catalog) -> model::Catalog<'_> {
    match cache {
        Catalog::Idle | Catalog::Loading => model::Catalog::Loading,
        Catalog::Failed => model::Catalog::Failed,
        Catalog::Loaded { tags, .. } => model::Catalog::Loaded(tags),
    }
}

/// Applies `action` to the model and performs the effects it returns, in order.
///
/// Use leaves the slug in the handoff and goes back, so the host adopts it in its `Screen::on_return`; the app keeps the dictionary up instead, since its host adopts through a callback.
pub(crate) fn dispatch(view: &mut OracleTagsView, action: Action, cx: &mut Context<'_>) {
    for effect in view.model.update(action) {
        match effect {
            Effect::OfferHint(offer) => {
                hint::once(cx, offer.key());
            }
            Effect::EnsureCatalog => ensure_catalog(cx),
            Effect::AdoptTag(slug) => {
                cx.shared.caches.handoff.oracle_tag = Some(slug);
                cx.back();
            }
            Effect::Navigate(Destination::Previous) => cx.back(),
            Effect::Navigate(Destination::Examples(slug)) => otag_examples::open(cx, &slug),
            Effect::Toast(toast) => cx.show(toast),
        }
    }
}

/// The catalog's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// The whole oracle tag catalog.
    Catalog,
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
        }
    }
}

/// The catalog's results.
#[derive(Debug)]
pub(crate) enum Update {
    Catalog(Vec<OracleTag>),
    CatalogFailed,
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::OracleTags(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::OracleTags(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut OracleTagsView> {
    match screen {
        Screen::OracleTags(view) => Some(view),
        _ => None,
    }
}

/// Fetches the catalog when it is due: never fetched, failed, or a day old. A fetch already running, or fresh tags, make this do nothing. Tags already loaded stay while a refetch runs.
pub(crate) fn ensure_catalog(cx: &mut Context<'_>) {
    let caches = &mut cx.shared.caches;
    if !caches.catalog_due(Instant::now()) {
        return;
    }
    match &mut caches.oracle_tags {
        Catalog::Loaded { refreshing, .. } => *refreshing = true,
        other => *other = Catalog::Loading,
    }
    cx.request(Request::Catalog.into());
}

/// Opens the dictionary over the screen that wants a tag, clearing any tag left from an earlier Use. See the module doc for how the host takes the tag.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.shared.caches.handoff.oracle_tag = None;
    let mut view = OracleTagsView::default();
    dispatch(&mut view, Action::Load, cx);
    cx.push(Screen::OracleTags(Box::new(view)));
}

/// Shows the dictionary's hint again, for its "?".
pub(crate) fn reopen_hint(cx: &mut Context<'_>) {
    hint::reopen(cx, HINT_ORACLE_TAG_DICTIONARY);
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request::Catalog = request;
    // The catalog is public, so it needs no session; any failure reads the same to the user.
    let update = match account.client.fetch_oracle_tags().await {
        Ok(tags) => Update::Catalog(tags),
        Err(_) => Update::CatalogFailed,
    };
    Some(update.into())
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let catalog = &mut cx.shared.caches.oracle_tags;
    match update {
        Update::Catalog(tags) => {
            *catalog = Catalog::Loaded {
                tags,
                fetched_at: Instant::now(),
                refreshing: false,
            };
        }
        Update::CatalogFailed => {
            match &mut *catalog {
                Catalog::Loaded { refreshing, .. } => *refreshing = false,
                other => *other = Catalog::Failed,
            }
            let failed = matches!(catalog, Catalog::Failed);
            if let Some(view) = nav.find_mut(pick)
                && failed
            {
                dispatch(view, Action::CatalogFailed, cx);
            }
        }
    }
}
