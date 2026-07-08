//! Per-deck parking for the add screen's search stack.
//!
//! Leaving the add screen parks the live stack under its deck id; returning
//! to that deck with the same filter restores it instead of refetching, so
//! every deck resumes exactly where its swiping left off. Restart loses the
//! cache, but durable skips mean a fresh fetch serves the right cards anyway.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::{Card, search_card::card_filter::builder::CardQueryBuilder};

use crate::inbound::screens::deck::card::components::action_history::AddAction;

/// Most-recently-used decks kept parked; the oldest is evicted beyond this.
/// Matches the server's MAX_DECKS_PER_USER, so in practice every deck a user
/// can own stays parked — trimmed parks are small (~80 cards) and the cache
/// only ever holds decks visited this app session.
const MAX_PARKED_DECKS: usize = 20;

/// Swiped-past cards kept when parking — the undo depth after returning.
const PARKED_BEHIND_CARDS: usize = 50;

/// A parked search stack for one deck.
pub struct ParkedStack {
    /// The card list (behind-cursor zone trimmed on park).
    pub cards: Vec<Card>,
    /// Cursor position within `cards`.
    pub index: usize,
    /// Undo history (trimmed to the kept behind-cursor depth).
    pub history: Vec<AddAction>,
    /// Filter that produced the stack; a change invalidates the park.
    pub filter: CardQueryBuilder,
    /// Server pagination offset (not derivable from `cards` after trimming).
    pub offset: u32,
    /// Whether pagination had already run dry.
    pub exhausted: bool,
}

/// App-scoped MRU cache of parked stacks, one per deck.
#[derive(Clone, Copy)]
pub struct AddStackCache {
    entries: Signal<Vec<(Uuid, ParkedStack)>>,
}

/// Creates the cache (provided as context in `spawn_upkeeper`).
pub fn use_add_stack_cache() -> AddStackCache {
    AddStackCache {
        entries: use_signal(Vec::new),
    }
}

impl AddStackCache {
    /// Parks a deck's stack: trims the swiped-past zone to the undo depth,
    /// fronts the deck in the MRU order, and evicts beyond the cap.
    pub fn park(&mut self, deck_id: Uuid, mut parked: ParkedStack) {
        let cut = parked.index.saturating_sub(PARKED_BEHIND_CARDS);
        if cut > 0 {
            parked.cards.drain(..cut);
            parked.index -= cut;
        }
        // History deeper than the kept zone has no card to step back onto.
        let excess = parked.history.len().saturating_sub(parked.index);
        parked.history.drain(..excess);

        let mut entries = self.entries.write();
        entries.retain(|(id, _)| *id != deck_id);
        entries.insert(0, (deck_id, parked));
        entries.truncate(MAX_PARKED_DECKS);
    }

    /// Removes and returns a deck's parked stack, if any.
    pub fn take(&mut self, deck_id: Uuid) -> Option<ParkedStack> {
        let mut entries = self.entries.write();
        let pos = entries.iter().position(|(id, _)| *id == deck_id)?;
        Some(entries.remove(pos).1)
    }
}
