//! Contained swipe-stack state: cards, cursor, undo history, and the enter
//! animation, kept in lockstep by construction.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;

use crate::inbound::components::interactions::swipe::STACK_DEPTH;
use crate::inbound::components::interactions::swipe::direction::Direction;

/// A swipeable card stack: the fetched cards, the top-card cursor, the undo
/// history, and the enter-animation direction. A `Copy` handle over signals,
/// so closures capture it freely while readers keep per-signal reactivity.
///
/// The Remove screen's cycling stack wraps around a fixed list and mutates it
/// in place; the `*_wrapping` methods serve it.
pub(crate) struct CardStack<A: 'static> {
    cards: Signal<Vec<Card>>,
    index: Signal<usize>,
    history: Signal<Vec<A>>,
    entering: Signal<Option<Direction>>,
}

// Manual impls: `derive` would demand `A: Copy`, but the handle is Copy
// regardless of the action type (all fields are signals).
impl<A: 'static> Clone for CardStack<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: 'static> Copy for CardStack<A> {}

/// Creates a stack owned by the calling component.
pub(crate) fn use_card_stack<A: 'static>() -> CardStack<A> {
    CardStack {
        cards: use_signal(Vec::new),
        index: use_signal(|| 0),
        history: use_signal(Vec::new),
        entering: use_signal(|| None),
    }
}

impl<A: 'static> CardStack<A> {
    // == reads ==

    /// Clones the full card list (reactive read).
    pub(crate) fn cards(&self) -> Vec<Card> {
        (self.cards)()
    }

    /// True when the stack holds no cards (reactive read).
    pub(crate) fn is_empty(&self) -> bool {
        self.cards.read().is_empty()
    }

    /// Cursor position (reactive read).
    pub(crate) fn index(&self) -> usize {
        (self.index)()
    }

    /// The `STACK_DEPTH` cards from the cursor: `SwipeStack`'s render window.
    pub(crate) fn window(&self) -> Vec<Card> {
        self.cards()
            .into_iter()
            .skip(self.index())
            .take(STACK_DEPTH)
            .collect()
    }

    /// Card under the cursor (linear stacks).
    pub(crate) fn current(&self) -> Option<Card> {
        self.cards.read().get(self.index()).cloned()
    }

    /// Card under the cursor with wrap-around (cycling stacks).
    pub(crate) fn current_wrapping(&self) -> Option<Card> {
        let cards = self.cards.read();
        // `None` for an empty stack, which has no remainder.
        let i = self.index().checked_rem(cards.len())?;
        cards.get(i).cloned()
    }

    /// The enter-animation signal, passed straight through as `SwipeStack`'s
    /// `entering` prop.
    pub(crate) fn entering(&self) -> Signal<Option<Direction>> {
        self.entering
    }

    // == contents ==

    /// Swaps in a new card list and rewinds the cursor. History survives,
    /// undo semantics across a re-filter are the caller's call (see `rewind`).
    pub(crate) fn replace(&mut self, cards: Vec<Card>) {
        self.cards.set(cards);
        self.index.set(0);
    }

    /// Rewinds the cursor and drops the undo history and any primed
    /// animation. The card list is untouched.
    pub(crate) fn rewind(&mut self) {
        self.index.set(0);
        self.history.write().clear();
        self.entering.set(None);
    }

    /// Removes the card at the cursor; the next card slides into position
    /// (cursor unchanged). No-op when the cursor is past the end.
    pub(crate) fn remove_current(&mut self) {
        let idx = (self.index)();
        if idx < self.cards.read().len() {
            self.cards.write().remove(idx);
        }
    }

    /// Re-inserts a card at the cursor (undo of `remove_current`).
    pub(crate) fn insert_current(&mut self, card: Card) {
        let idx = (self.index)().min(self.cards.read().len());
        self.cards.write().insert(idx, card);
    }

    // == cursor ==

    /// Moves the cursor forward with wrap-around (cycling stacks).
    pub(crate) fn advance_wrapping(&mut self) {
        let len = self.cards.read().len();
        // The cursor stays put on an empty stack, which has no remainder.
        if let Some(next) = (self.index)()
            .checked_add(1)
            .and_then(|next| next.checked_rem(len))
        {
            self.index.set(next);
        }
    }

    /// Steps the cursor back with wrap-around (cycling stacks). Returns false
    /// when the stack is empty.
    pub(crate) fn retreat_wrapping(&mut self) -> bool {
        let Some(last) = self.cards.read().len().checked_sub(1) else {
            return false;
        };
        let idx = (self.index)();
        self.index.set(idx.checked_sub(1).unwrap_or(last));
        true
    }

    // == history + animation ==

    /// Pushes a committed swipe onto the undo history.
    pub(crate) fn record(&mut self, action: A) {
        self.history.write().push(action);
    }

    /// Pops the most recent swipe, or None when there's nothing to undo.
    pub(crate) fn pop_action(&mut self) -> Option<A> {
        self.history.write().pop()
    }

    /// Primes the enter animation for the next top card.
    pub(crate) fn prime_entering(&mut self, direction: Direction) {
        self.entering.set(Some(direction));
    }

    /// Cancels a primed enter animation (aborted undo).
    pub(crate) fn cancel_entering(&mut self) {
        self.entering.set(None);
    }
}
