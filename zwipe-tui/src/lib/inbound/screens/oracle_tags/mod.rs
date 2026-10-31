//! The oracle tag dictionary, and the app-wide oracle tag catalog it reads.
//!
//! # The catalog
//!
//! `shared.caches.oracle_tags` holds the catalog for the whole launch, as the app's `CatalogCache` does. [`ensure_catalog`] fetches it with `fetch_oracle_tags` when it is missing, failed or a day old; it is single flight, and a refetch keeps the old tags on screen until the new ones land (stale while revalidate). Its result is written to the cache whether or not the dictionary is open. Every screen that shows tag descriptions calls [`ensure_catalog`] when it opens.
//!
//! # The dictionary
//!
//! A search box, a letter rail A to Z (plus `#` when some slug starts with something else), and the tags under the letter or matching the search, each with its description and parents. Each tag offers Examples (the cards that carry it) and Use.
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

use zwipe_core::domain::card::oracle_tag::{self, OracleTag};
use zwipe_core::domain::user::models::hints::HINT_ORACLE_TAG_DICTIONARY;

use super::Screen;
use super::home::cache::Catalog;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::account::Account;

/// The dictionary's title, and its hint's.
pub(crate) const TITLE: &str = "Oracle tags";

/// The search box's placeholder.
pub(crate) const PLACEHOLDER: &str = "Search tags or descriptions";

/// What the dictionary says when the catalog failed with nothing cached.
pub(crate) const FAILED: &str = "Couldn't load the dictionary. Pull back and try again.";

/// The toast raised once when the catalog fails while the dictionary is open.
pub(crate) const FAILED_TOAST: &str = "Couldn't load the oracle-tag dictionary";

/// A search shows at most this many tags, as the app's `SEARCH_RESULT_CAP`.
pub(crate) const SEARCH_RESULT_CAP: usize = 100;

/// The bucket for slugs that start with anything but a letter.
pub(crate) const OTHER: char = '#';

/// The dictionary's state. The tags live in the catalog, so this holds only the search, the letter and the selected row.
#[derive(Debug)]
pub(crate) struct OracleTagsView {
    pub(crate) search: TextInput,
    /// Keys go to the search box rather than the list.
    pub(crate) typing: bool,
    /// The letter browsed while not searching, lowercase, or [`OTHER`].
    pub(crate) letter: char,
    /// The selected row in the list on screen.
    pub(crate) selected: usize,
    /// The failure toast has been raised for this visit.
    pub(crate) failure_shown: bool,
}

impl Default for OracleTagsView {
    /// Browsing A with an empty search.
    fn default() -> Self {
        Self {
            search: TextInput::default().with_placeholder(PLACEHOLDER),
            typing: false,
            letter: 'a',
            selected: 0,
            failure_shown: false,
        }
    }
}

impl OracleTagsView {
    /// The search as it matches: trimmed and lowercased. Empty means browsing by letter.
    pub(crate) fn query(&self) -> String {
        self.search.value().trim().to_lowercase()
    }

    /// The tags on screen: the search's ranked matches up to [`SEARCH_RESULT_CAP`], or the letter's tags by slug.
    pub(crate) fn rows(&self, tags: &[OracleTag]) -> Vec<OracleTag> {
        let query = self.query();
        if query.is_empty() {
            let mut rows: Vec<OracleTag> = tags
                .iter()
                .filter(|t| bucket_of(&t.slug) == self.letter)
                .cloned()
                .collect();
            rows.sort_by(|a, b| a.slug.cmp(&b.slug));
            rows
        } else {
            let mut rows = oracle_tag::search_oracle_tags(tags, &query);
            rows.truncate(SEARCH_RESULT_CAP);
            rows
        }
    }

    /// Clears the search and browses the letter `step` places along the rail, stopping at either end.
    pub(crate) fn step_letter(&mut self, tags: &[OracleTag], step: Step) {
        let letters = letters(tags);
        let at = letters.iter().position(|&l| l == self.letter).unwrap_or(0);
        let to = match step {
            Step::Back => at.saturating_sub(1),
            Step::Forward => at.saturating_add(1).min(letters.len().saturating_sub(1)),
        };
        self.letter = letters.get(to).copied().unwrap_or('a');
        self.search.set_value("");
        self.selected = 0;
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

/// The letter a slug is browsed under: its first character lowercased when it is a letter, else [`OTHER`].
pub(crate) fn bucket_of(slug: &str) -> char {
    match slug.chars().next() {
        Some(c) if c.is_ascii_alphabetic() => c.to_ascii_lowercase(),
        _ => OTHER,
    }
}

/// The rail: a to z, then [`OTHER`] only when some slug needs it.
pub(crate) fn letters(tags: &[OracleTag]) -> Vec<char> {
    let other = tags.iter().any(|t| bucket_of(&t.slug) == OTHER);
    ('a'..='z').chain(other.then_some(OTHER)).collect()
}

/// The catalog's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// The whole oracle tag catalog.
    Catalog,
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
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "its hosts, the deck's oracle tag picker and the card filter, come with the deck and card screens"
    )
)]
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.shared.caches.handoff.oracle_tag = None;
    ensure_catalog(cx);
    cx.push(Screen::OracleTags(Box::default()));
    hint::once(cx, HINT_ORACLE_TAG_DICTIONARY);
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
                && !view.failure_shown
            {
                view.failure_shown = true;
                cx.toast(ToastKind::Error, FAILED_TOAST.to_owned());
            }
        }
    }
}
