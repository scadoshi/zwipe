//! The stores the card screens share for the visit, zwipe-ui's, held in
//! root signals: each screen's filter per deck, Add's parked stacks and its
//! live search stack, the per-deck undo log and the adds still in flight.
//!
//! They live in the root because work outlives the screen that started it:
//! an add's answer lands after Add is gone and still records its undo entry.
//! Nothing here is saved; a reload forgets it, as an app restart does.

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_ui::stack::{AddAction, CardStack};
use zwipe_ui::stores::filters::FilterStore;
use zwipe_ui::stores::in_flight::InFlightAdds;
use zwipe_ui::stores::parked::AddStackCache;
use zwipe_ui::stores::undo::UndoStore;

/// The shared card stores. `Copy`; read with `stores.undo.read()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CardStores {
    /// Each card screen's filter per deck.
    pub filters: Signal<FilterStore>,
    /// Add's search stacks parked per deck.
    pub stacks: Signal<AddStackCache>,
    /// The per-deck undo log.
    pub undo: Signal<UndoStore>,
    /// Adds whose create call has not answered.
    pub in_flight: Signal<InFlightAdds>,
    /// Add's search stack, kept across visits so coming back to the same
    /// deck resumes mid-stack.
    pub add_stack: Signal<CardStack<AddAction>>,
    /// The filter that produced `add_stack`; `None` while a search is out.
    pub last_search: Signal<Option<CardQueryBuilder>>,
}

/// Provides [`CardStores`] to everything under the root. Call once, in the
/// root.
pub fn use_card_stores() {
    let stores = CardStores {
        filters: use_signal(FilterStore::default),
        stacks: use_signal(AddStackCache::default),
        undo: use_signal(UndoStore::default),
        in_flight: use_signal(InFlightAdds::default),
        add_stack: use_signal(CardStack::default),
        last_search: use_signal(|| None),
    };
    use_context_provider(|| stores);
}
