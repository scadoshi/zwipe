//! Per-(screen, deck) filter memory.
//!
//! Every swipe screen on every deck remembers its own filter for the session:
//! each screen owns a local `Signal<CardQueryBuilder>` (provided as context so
//! the filter modules bind to it), initialized from here on mount and parked
//! back on drop. Independent contexts replace the old shared filter and its
//! "move it across unless it looks like a default" guesswork — cross-screen
//! contamination is structurally impossible. In-memory only: an app restart
//! forgets. Plan: `context/plans/filter_persistence.md`.

use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;

/// Which screen a remembered filter belongs to. The add screen's two sources
/// are separate scopes — toggling parks one and restores the other.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterScope {
    /// Deck cards screen (view).
    Cards,
    /// Remove cards screen.
    Remove,
    /// Add screen, Search source.
    AddSearch,
    /// Add screen, Maybeboard source.
    AddMaybeboard,
}

/// App-scoped store of remembered filters, keyed by (scope, deck). Unbounded
/// on purpose: builders are tiny, and the map only holds decks visited this
/// session.
#[derive(Clone, Copy)]
pub struct FilterStore {
    entries: Signal<HashMap<(FilterScope, Uuid), CardQueryBuilder>>,
}

/// Creates the store (provided as context in `spawn_upkeeper`).
pub fn use_filter_store() -> FilterStore {
    FilterStore {
        entries: use_signal(HashMap::new),
    }
}

impl FilterStore {
    /// The filter last parked for this (scope, deck), if any.
    pub fn restore(&self, scope: FilterScope, deck_id: Uuid) -> Option<CardQueryBuilder> {
        self.entries.peek().get(&(scope, deck_id)).cloned()
    }

    /// Remembers a filter for this (scope, deck), replacing any prior one.
    pub fn park(&mut self, scope: FilterScope, deck_id: Uuid, filter: CardQueryBuilder) {
        self.entries.write().insert((scope, deck_id), filter);
    }
}
