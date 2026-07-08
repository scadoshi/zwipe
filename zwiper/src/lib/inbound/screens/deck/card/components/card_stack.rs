//! Contained swipe-stack state: cards, cursor, undo history, and the enter
//! animation, kept in lockstep by construction.

use dioxus::prelude::*;

use crate::inbound::{
    components::interactions::swipe::{STACK_DEPTH, direction::Direction},
    screens::deck::card::components::action_history::StackAction,
};
use zwipe_core::domain::card::Card;

/// A swipeable card stack: the fetched cards, the top-card cursor, the undo
/// history, and the enter-animation direction. A `Copy` handle over signals,
/// so closures capture it freely while readers keep per-signal reactivity.
///
/// Two cursor disciplines share this type: linear stacks (the add screen's
/// search source) move forward through a paginated list, while cycling stacks
/// (maybeboard source, remove screen) wrap around a fixed list and mutate it
/// in place. The `*_wrapping` methods serve the latter.
pub struct CardStack<A: 'static> {
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

/// Creates a stack owned by the calling component (screen-local stacks). The
/// add screen's search stack is instead created in `spawn_upkeeper` and
/// provided as context, so it outlives the screen.
pub fn use_card_stack<A: 'static>() -> CardStack<A> {
    CardStack {
        cards: use_signal(Vec::new),
        index: use_signal(|| 0),
        history: use_signal(Vec::new),
        entering: use_signal(|| None),
    }
}

impl<A: 'static> CardStack<A> {
    // ── reads ────────────────────────────────────────────────────────────

    /// Clones the full card list (reactive read).
    pub fn cards(&self) -> Vec<Card> {
        (self.cards)()
    }

    /// Number of cards (reactive read).
    pub fn len(&self) -> usize {
        self.cards.read().len()
    }

    /// True when the stack holds no cards (reactive read).
    pub fn is_empty(&self) -> bool {
        self.cards.read().is_empty()
    }

    /// `is_empty` without subscribing the caller.
    pub fn peek_is_empty(&self) -> bool {
        self.cards.peek().is_empty()
    }

    /// Cursor position (reactive read).
    pub fn index(&self) -> usize {
        (self.index)()
    }

    /// The `STACK_DEPTH` cards from the cursor — `SwipeStack`'s render window.
    pub fn window(&self) -> Vec<Card> {
        self.cards()
            .into_iter()
            .skip(self.index())
            .take(STACK_DEPTH)
            .collect()
    }

    /// Card under the cursor (linear stacks).
    pub fn current(&self) -> Option<Card> {
        self.cards.read().get(self.index()).cloned()
    }

    /// Card under the cursor with wrap-around (cycling stacks).
    pub fn current_wrapping(&self) -> Option<Card> {
        let cards = self.cards.read();
        if cards.is_empty() {
            return None;
        }
        cards.get(self.index() % cards.len()).cloned()
    }

    /// The enter-animation signal, passed straight through as `SwipeStack`'s
    /// `entering` prop.
    pub fn entering(&self) -> Signal<Option<Direction>> {
        self.entering
    }

    // ── contents ─────────────────────────────────────────────────────────

    /// Swaps in a new card list and rewinds the cursor. History survives —
    /// undo semantics across a re-filter are the caller's call (see `rewind`).
    pub fn replace(&mut self, cards: Vec<Card>) {
        self.cards.set(cards);
        self.index.set(0);
    }

    /// Appends a page of cards (pagination).
    pub fn append(&mut self, more: Vec<Card>) {
        self.cards.write().extend(more);
    }

    /// Rewinds the cursor and drops the undo history and any primed
    /// animation. The card list is untouched.
    pub fn rewind(&mut self) {
        self.index.set(0);
        self.history.write().clear();
        self.entering.set(None);
    }

    /// Clears everything: cards, cursor, history, animation.
    pub fn reset(&mut self) {
        self.cards.set(Vec::new());
        self.index.set(0);
        self.history.write().clear();
        self.entering.set(None);
    }

    /// Snapshot for parking (peeked — no subscriptions).
    pub fn park_state(&self) -> (Vec<Card>, usize, Vec<A>)
    where
        A: Clone,
    {
        (
            self.cards.peek().clone(),
            *self.index.peek(),
            self.history.peek().clone(),
        )
    }

    /// Restores a parked snapshot (un-park).
    pub fn restore(&mut self, cards: Vec<Card>, index: usize, history: Vec<A>) {
        self.index.set(index.min(cards.len()));
        self.cards.set(cards);
        self.history.set(history);
        self.entering.set(None);
    }

    /// Removes the card at the cursor; the next card slides into position
    /// (cursor unchanged). No-op when the cursor is past the end.
    pub fn remove_current(&mut self) {
        let idx = (self.index)();
        if idx < self.cards.read().len() {
            self.cards.write().remove(idx);
        }
    }

    /// Re-inserts a card at the cursor (undo of `remove_current`).
    pub fn insert_current(&mut self, card: Card) {
        let idx = (self.index)().min(self.cards.read().len());
        self.cards.write().insert(idx, card);
    }

    // ── cursor ───────────────────────────────────────────────────────────

    /// Moves the cursor forward one card (linear stacks). The cursor may land
    /// one past the end: the just-swiped last card leaves the window and undo
    /// stays aligned with the swipe that committed. Returns false only when
    /// already past the end.
    pub fn advance(&mut self) -> bool {
        let next = (self.index)() + 1;
        if next <= self.cards.read().len() {
            self.index.set(next);
            true
        } else {
            false
        }
    }

    /// Moves the cursor forward with wrap-around (cycling stacks).
    pub fn advance_wrapping(&mut self) {
        let len = self.cards.read().len();
        if len > 0 {
            self.index.set(((self.index)() + 1) % len);
        }
    }

    /// Steps the cursor back with wrap-around (cycling stacks). Returns false
    /// when the stack is empty.
    pub fn retreat_wrapping(&mut self) -> bool {
        let len = self.cards.read().len();
        if len == 0 {
            return false;
        }
        let idx = (self.index)();
        self.index.set(if idx == 0 { len - 1 } else { idx - 1 });
        true
    }

    // ── history + animation ──────────────────────────────────────────────

    /// Pushes a committed swipe onto the undo history.
    pub fn record(&mut self, action: A) {
        self.history.write().push(action);
    }

    /// Pops the most recent swipe, or None when there's nothing to undo.
    pub fn pop_action(&mut self) -> Option<A> {
        self.history.write().pop()
    }

    /// Reverses a `step_back` after a failed async undo: re-records the
    /// action, restores the cursor, cancels the primed animation.
    pub fn unwind_undo(&mut self, action: A) {
        self.history.write().push(action);
        self.index.set((self.index)() + 1);
        self.entering.set(None);
    }

    /// Primes the enter animation for the next top card.
    pub fn prime_entering(&mut self, direction: Direction) {
        self.entering.set(Some(direction));
    }

    /// Cancels a primed enter animation (aborted undo).
    pub fn cancel_entering(&mut self) {
        self.entering.set(None);
    }
}

impl<A: StackAction + 'static> CardStack<A> {
    /// Linear undo cursor move: steps back one and primes the enter animation
    /// from the action's exit direction. Returns false at the first card —
    /// callers should `record` the action back.
    pub fn step_back(&mut self, action: &A) -> bool {
        let idx = (self.index)();
        if idx == 0 {
            return false;
        }
        self.index.set(idx - 1);
        self.entering.set(Some(action.exited()));
        true
    }
}
