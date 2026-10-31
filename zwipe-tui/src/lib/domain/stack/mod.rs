//! The swipe stacks the Add and Remove screens share, ported from the app's `CardStack` and its action models.
//!
//! Two cursor disciplines share [`CardStack`]. A linear stack (Add's search source) walks forward through a paged list and keeps swiped cards behind the cursor, so undo steps back onto the card it undoes. A cycling stack (Add's maybeboard source, Remove) wraps around a fixed list and drops cards from it, so its actions carry the card for undo to put back.

use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;

/// Most cards a search stack holds before the user must refresh.
pub(crate) const MAX_CARDS_IN_STACK: usize = 500;

/// From this many cards, a toast warns of the cap every [`STACK_WARNING_EVERY`] cards.
pub(crate) const CARDS_WARNING_THRESHOLD: usize = 400;

/// How often the cap warning repeats once past [`CARDS_WARNING_THRESHOLD`].
pub(crate) const STACK_WARNING_EVERY: usize = 100;

/// The cards a swipe surface shows at once, the top one taking input.
pub(crate) const STACK_DEPTH: usize = 10;

/// A swipe direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// What every stack action shares: the side its card left by, so undo can bring it back from there.
pub(crate) trait StackAction {
    fn exited(&self) -> Direction;
}

/// A swipe on Add's search stack. It carries no card: the card stays behind the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AddAction {
    /// Left: skipped for a week.
    Skip,
    /// Right: added to the deck.
    Add,
    /// Up: added to the maybeboard.
    Maybe,
}

impl StackAction for AddAction {
    fn exited(&self) -> Direction {
        match self {
            Self::Skip => Direction::Left,
            Self::Add => Direction::Right,
            Self::Maybe => Direction::Up,
        }
    }
}

/// A swipe on Add's maybeboard stack.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MaybeboardAction {
    /// Left: cycled past.
    Skip,
    /// Right: promoted to the deck, which drops it from the stack.
    Promote { card: Box<Card> },
}

impl StackAction for MaybeboardAction {
    fn exited(&self) -> Direction {
        match self {
            Self::Skip => Direction::Left,
            Self::Promote { .. } => Direction::Right,
        }
    }
}

/// A swipe on the Remove stack.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RemoveAction {
    /// Left: kept, cycled past.
    Keep,
    /// Right: every copy removed from the deck.
    Remove {
        card: Box<Card>,
        /// The board it sat on, so undo puts it back there.
        board: Board,
        /// The copies the deck held, so undo restores all of them.
        quantity: i32,
    },
    /// Up: moved between the main board and the maybeboard.
    MoveBoard {
        card: Box<Card>,
        from: Board,
        to: Board,
    },
}

impl StackAction for RemoveAction {
    fn exited(&self) -> Direction {
        match self {
            Self::Keep => Direction::Left,
            Self::Remove { .. } => Direction::Right,
            Self::MoveBoard { .. } => Direction::Up,
        }
    }
}

/// A swipeable stack: its cards, the cursor, the undo history, and the side the top card is coming back from after an undo.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CardStack<A> {
    cards: Vec<Card>,
    index: usize,
    history: Vec<A>,
    entering: Option<Direction>,
}

impl<A> Default for CardStack<A> {
    fn default() -> Self {
        Self {
            cards: Vec::new(),
            index: 0,
            history: Vec::new(),
            entering: None,
        }
    }
}

impl<A> CardStack<A> {
    pub(crate) fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub(crate) fn len(&self) -> usize {
        self.cards.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn history(&self) -> &[A] {
        &self.history
    }

    /// The side the top card is coming back from after an undo, until the next swipe or key.
    pub(crate) fn entering(&self) -> Option<Direction> {
        self.entering
    }

    /// Up to [`STACK_DEPTH`] cards from the cursor.
    pub(crate) fn window(&self) -> &[Card] {
        let end = self.index.saturating_add(STACK_DEPTH).min(self.cards.len());
        self.cards.get(self.index..end).unwrap_or_default()
    }

    /// The card under the cursor on a linear stack; `None` once the cursor is past the end.
    pub(crate) fn current(&self) -> Option<&Card> {
        self.cards.get(self.index)
    }

    /// The card under the cursor on a cycling stack, wrapping around; `None` only when empty.
    pub(crate) fn current_wrapping(&self) -> Option<&Card> {
        let len = self.cards.len();
        if len == 0 {
            return None;
        }
        self.cards.get(self.index % len)
    }

    /// Swaps in new cards and moves the cursor to the first. History stays.
    pub(crate) fn replace(&mut self, cards: Vec<Card>) {
        self.cards = cards;
        self.index = 0;
    }

    /// Appends a page of cards.
    pub(crate) fn append(&mut self, more: Vec<Card>) {
        self.cards.extend(more);
    }

    /// Moves the cursor to the first card and drops the history; the cards stay.
    pub(crate) fn rewind(&mut self) {
        self.index = 0;
        self.history.clear();
        self.entering = None;
    }

    /// Drops everything.
    pub(crate) fn reset(&mut self) {
        self.cards.clear();
        self.index = 0;
        self.history.clear();
        self.entering = None;
    }

    /// The cards, cursor and history, for parking.
    pub(crate) fn park_state(&self) -> (Vec<Card>, usize, Vec<A>)
    where
        A: Clone,
    {
        (self.cards.clone(), self.index, self.history.clone())
    }

    /// Restores a parked stack, the cursor kept within the cards.
    pub(crate) fn restore(&mut self, cards: Vec<Card>, index: usize, history: Vec<A>) {
        self.index = index.min(cards.len());
        self.cards = cards;
        self.history = history;
        self.entering = None;
    }

    /// Drops the card under the cursor so the next one slides in. Does nothing past the end.
    pub(crate) fn remove_current(&mut self) {
        if self.index < self.cards.len() {
            self.cards.remove(self.index);
        }
    }

    /// Puts a card back under the cursor, the undo of [`CardStack::remove_current`].
    pub(crate) fn insert_current(&mut self, card: Card) {
        let at = self.index.min(self.cards.len());
        self.cards.insert(at, card);
    }

    /// Swaps the card under the cursor for another, such as a chosen printing. Does nothing past the end.
    pub(crate) fn replace_current(&mut self, card: Card) {
        if let Some(slot) = self.cards.get_mut(self.index) {
            *slot = card;
        }
    }

    /// Moves a linear stack's cursor forward; it may land one past the end. Returns false when it already was there.
    pub(crate) fn advance(&mut self) -> bool {
        let next = self.index.saturating_add(1);
        if next <= self.cards.len() {
            self.index = next;
            true
        } else {
            false
        }
    }

    /// Moves a cycling stack's cursor forward, wrapping to the first card.
    pub(crate) fn advance_wrapping(&mut self) {
        let len = self.cards.len();
        if len > 0 {
            self.index = self.index.saturating_add(1) % len;
        }
    }

    /// Moves a cycling stack's cursor back, wrapping to the last card. Returns false when the stack is empty.
    pub(crate) fn retreat_wrapping(&mut self) -> bool {
        let len = self.cards.len();
        if len == 0 {
            return false;
        }
        self.index = match self.index.checked_sub(1) {
            Some(previous) => previous.min(len.saturating_sub(1)),
            None => len.saturating_sub(1),
        };
        true
    }

    pub(crate) fn record(&mut self, action: A) {
        self.history.push(action);
    }

    /// The newest swipe, taken off the history.
    pub(crate) fn pop_action(&mut self) -> Option<A> {
        self.history.pop()
    }

    /// Reverses a [`CardStack::step_back`] after its undo failed: records the action again and moves the cursor forward.
    pub(crate) fn unwind_undo(&mut self, action: A) {
        self.history.push(action);
        self.index = self.index.saturating_add(1);
        self.entering = None;
    }

    pub(crate) fn prime_entering(&mut self, direction: Direction) {
        self.entering = Some(direction);
    }

    pub(crate) fn cancel_entering(&mut self) {
        self.entering = None;
    }
}

impl<A: StackAction> CardStack<A> {
    /// Steps a linear stack's cursor back onto the card `action` swiped and marks it as coming back from the side it left. Returns false at the first card.
    pub(crate) fn step_back(&mut self, action: &A) -> bool {
        let Some(previous) = self.index.checked_sub(1) else {
            return false;
        };
        self.index = previous;
        self.entering = Some(action.exited());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::card;

    fn stack(names: &[&str]) -> CardStack<AddAction> {
        let mut stack = CardStack::default();
        stack.replace(names.iter().map(|n| card(n)).collect());
        stack
    }

    fn name(card: Option<&Card>) -> Option<&str> {
        card.map(|c| c.scryfall_data.name.as_str())
    }

    #[test]
    fn a_linear_stack_walks_one_past_the_end_and_steps_back() {
        let mut stack = stack(&["A", "B"]);
        assert!(stack.advance());
        stack.record(AddAction::Add);
        assert!(stack.advance());
        stack.record(AddAction::Skip);
        assert_eq!(stack.current(), None, "one past the end");
        assert!(!stack.advance(), "no further");

        let action = stack.pop_action().unwrap();
        assert!(stack.step_back(&action));
        assert_eq!(name(stack.current()), Some("B"));
        assert_eq!(stack.entering(), Some(Direction::Left));

        stack.unwind_undo(action);
        assert_eq!(stack.current(), None, "a failed undo goes forward again");
        assert_eq!(stack.history(), [AddAction::Add, AddAction::Skip]);
        assert_eq!(stack.entering(), None);
    }

    #[test]
    fn step_back_refuses_at_the_first_card() {
        let mut stack = stack(&["A"]);
        assert!(!stack.step_back(&AddAction::Add));
        assert_eq!(stack.index(), 0);
    }

    #[test]
    fn a_cycling_stack_wraps_both_ways() {
        let mut stack: CardStack<RemoveAction> = CardStack::default();
        stack.replace(vec![card("A"), card("B"), card("C")]);
        stack.advance_wrapping();
        stack.advance_wrapping();
        stack.advance_wrapping();
        assert_eq!(name(stack.current_wrapping()), Some("A"));
        assert!(stack.retreat_wrapping());
        assert_eq!(name(stack.current_wrapping()), Some("C"));
        assert!(!CardStack::<RemoveAction>::default().retreat_wrapping());
    }

    #[test]
    fn removing_and_reinserting_keeps_the_cursor() {
        let mut stack: CardStack<RemoveAction> = CardStack::default();
        stack.replace(vec![card("A"), card("B")]);
        let a = stack.current_wrapping().unwrap().clone();
        stack.remove_current();
        assert_eq!(name(stack.current_wrapping()), Some("B"));
        stack.insert_current(a);
        assert_eq!(name(stack.current_wrapping()), Some("A"));
        stack.replace_current(card("Z"));
        assert_eq!(name(stack.current_wrapping()), Some("Z"));
    }

    #[test]
    fn the_window_is_ten_deep_from_the_cursor() {
        let names: Vec<String> = (0..15).map(|i| i.to_string()).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut stack = stack(&refs);
        assert_eq!(stack.window().len(), STACK_DEPTH);
        for _ in 0..12 {
            stack.advance();
        }
        assert_eq!(stack.window().len(), 3);
    }

    #[test]
    fn restore_clamps_the_cursor_and_rewind_keeps_cards() {
        let mut stack: CardStack<AddAction> = CardStack::default();
        stack.restore(vec![card("A")], 9, vec![AddAction::Skip]);
        assert_eq!(stack.index(), 1);
        stack.rewind();
        assert_eq!(
            (stack.index(), stack.len(), stack.history().len()),
            (0, 1, 0)
        );
        stack.reset();
        assert!(stack.is_empty());
    }

    #[test]
    fn each_action_names_its_exit_side() {
        assert_eq!(AddAction::Maybe.exited(), Direction::Up);
        assert_eq!(MaybeboardAction::Skip.exited(), Direction::Left);
        assert_eq!(RemoveAction::Keep.exited(), Direction::Left);
    }
}
