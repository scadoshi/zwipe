//! The swipe stacks the Add and Remove screens share, ported from the app's `CardStack` and its action models.
//!
//! Two cursor disciplines share [`CardStack`]. A linear stack (Add's search source) walks forward through a paged list and keeps swiped cards behind the cursor, so undo steps back onto the card it undoes. A cycling stack (Add's maybeboard source, Remove) wraps around a fixed list and drops cards from it, so its actions carry the card for undo to put back.

use std::collections::HashSet;
use std::hash::BuildHasher;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;

/// Most cards a search stack holds before the user must refresh.
pub const MAX_CARDS_IN_STACK: usize = 500;

/// From this many cards, a toast warns of the cap every [`STACK_WARNING_EVERY`] cards.
pub const CARDS_WARNING_THRESHOLD: usize = 400;

/// How often the cap warning repeats once past [`CARDS_WARNING_THRESHOLD`].
pub const STACK_WARNING_EVERY: usize = 100;

/// The toast when a swipe stack runs out under the user's swipes, on every swipe screen. It is a
/// toast, not a swipe's confirmation, so it shows whatever the confirmation setting is, and it
/// takes the place of the last swipe's own line.
pub const NO_CARDS_LEFT: &str = "No cards left";

/// The note an Add stack's ghost card carries when the filter matches no card, so the empty pile
/// says why it is empty after the toast has gone.
pub const NO_CARDS_MATCH: &str = "No cards match your filters";

/// The cards a swipe surface shows at once, the top one taking input.
pub const STACK_DEPTH: usize = 10;

/// Whether a swipe stack whose top card is `top` still draws its ghost
/// under that card and keeps the cards under it hidden: until the top's id
/// is in `shown` (cards whose image has loaded and decoded, failed, or that
/// have no art), and while it is `painting` (its image came in while it was
/// the top, and has yet to have [`crate::timing::TOP_PAINT_GRACE`] to
/// paint). A card that loaded under the top shows at once when it comes up,
/// so a swipe never holds the ghost.
#[must_use]
pub fn ghost_held<S: BuildHasher>(
    top: Option<Uuid>,
    shown: &HashSet<Uuid, S>,
    painting: Option<Uuid>,
) -> bool {
    top.is_some_and(|id| !shown.contains(&id) || painting == Some(id))
}

/// A swipe direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Left.
    Left,
    /// Right.
    Right,
    /// Up.
    Up,
}

/// What every stack action shares: the side its card left by, so undo can bring it back from there.
pub trait StackAction {
    /// The side the card left by.
    fn exited(&self) -> Direction;
}

/// A swipe on Add's search stack. It carries no card: the card stays behind the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddAction {
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
pub enum MaybeboardAction {
    /// Left: cycled past.
    Skip,
    /// Right: promoted to the deck, which drops it from the stack.
    Promote {
        /// The promoted card, for undo to put back.
        card: Box<Card>,
    },
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
pub enum RemoveAction {
    /// Left: kept, cycled past; the card stays in the deck.
    Keep,
    /// Right: every copy removed from the deck.
    Remove {
        /// The removed card, for undo to put back.
        card: Box<Card>,
        /// The board it sat on, so undo puts it back there.
        board: Board,
        /// The copies the deck held, so undo restores all of them.
        quantity: i32,
        /// When it was starred as an MVP, so undo restores the star.
        mvp_at: Option<DateTime<Utc>>,
    },
    /// Up: moved between the main board and the maybeboard.
    MoveBoard {
        /// The moved card.
        card: Box<Card>,
        /// The board it left.
        from: Board,
        /// The board it went to.
        to: Board,
        /// Its star time when the move took it off the main board, which drops the star, so undo stars it again.
        starred: Option<DateTime<Utc>>,
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
pub struct CardStack<A> {
    cards: Vec<Card>,
    index: usize,
    history: Vec<A>,
    entering: Option<Direction>,
    /// How many of the first cards were swiped under an earlier search and
    /// kept by [`CardStack::replace_ahead`] for undo to reach.
    kept: usize,
}

impl<A> Default for CardStack<A> {
    fn default() -> Self {
        Self {
            cards: Vec::new(),
            index: 0,
            history: Vec::new(),
            entering: None,
            kept: 0,
        }
    }
}

impl<A> CardStack<A> {
    /// The stack's cards.
    #[must_use]
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// How many cards the stack holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Whether the stack holds no cards.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// The cursor.
    #[must_use]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Whether there is a swipe to undo.
    #[must_use]
    pub fn has_history(&self) -> bool {
        !self.history.is_empty()
    }

    /// The swipes so far, oldest first.
    #[cfg(test)]
    #[must_use]
    pub fn history(&self) -> &[A] {
        &self.history
    }

    /// The side the top card is coming back from after an undo, until the next swipe or key.
    #[must_use]
    pub fn entering(&self) -> Option<Direction> {
        self.entering
    }

    /// Up to [`STACK_DEPTH`] cards from the cursor.
    #[must_use]
    pub fn window(&self) -> &[Card] {
        let end = self.index.saturating_add(STACK_DEPTH).min(self.cards.len());
        self.cards.get(self.index..end).unwrap_or_default()
    }

    /// The card under the cursor on a linear stack; `None` once the cursor is past the end.
    #[must_use]
    pub fn current(&self) -> Option<&Card> {
        self.cards.get(self.index)
    }

    /// The card under the cursor on a cycling stack, wrapping around; `None` only when empty.
    #[must_use]
    pub fn current_wrapping(&self) -> Option<&Card> {
        self.cards.get(self.index.checked_rem(self.cards.len())?)
    }

    /// The card under the top one on a linear stack.
    #[must_use]
    pub fn next(&self) -> Option<&Card> {
        self.cards.get(self.index.checked_add(1)?)
    }

    /// The card under the top one on a cycling stack, wrapping around; `None` with fewer than two cards, since the top card would sit under itself.
    #[must_use]
    pub fn next_wrapping(&self) -> Option<&Card> {
        if self.cards.len() < 2 {
            return None;
        }
        let at = self.index.checked_add(1)?.checked_rem(self.cards.len())?;
        self.cards.get(at)
    }

    /// Swaps in new cards and moves the cursor to the first. History stays.
    pub fn replace(&mut self, cards: Vec<Card>) {
        self.cards = cards;
        self.index = 0;
        self.kept = 0;
    }

    /// Swaps the cards ahead of the cursor for new ones and moves the cursor to the first of them. The swiped cards stay behind it with the history, so undo still reaches them; a new card already among them is dropped.
    pub fn replace_ahead(&mut self, cards: Vec<Card>) {
        self.cards.truncate(self.index);
        self.kept = self.cards.len();
        let swiped: HashSet<Uuid> = self.cards.iter().map(|c| c.scryfall_data.id).collect();
        self.cards.extend(
            cards
                .into_iter()
                .filter(|c| !swiped.contains(&c.scryfall_data.id)),
        );
        self.entering = None;
    }

    /// Drops the cards ahead of the cursor, keeping the swiped ones and the history for undo.
    pub fn clear_ahead(&mut self) {
        self.replace_ahead(Vec::new());
    }

    /// Whether the live search put any card on the stack, as against cards kept from an earlier one by [`CardStack::replace_ahead`].
    #[must_use]
    pub fn has_served(&self) -> bool {
        self.cards.len() > self.kept
    }

    /// Appends a page of cards.
    pub fn append(&mut self, more: Vec<Card>) {
        self.cards.extend(more);
    }

    /// Moves the cursor to the first card and drops the history; the cards stay.
    pub fn rewind(&mut self) {
        self.index = 0;
        self.history.clear();
        self.entering = None;
    }

    /// Drops everything.
    pub fn reset(&mut self) {
        self.cards.clear();
        self.index = 0;
        self.history.clear();
        self.entering = None;
        self.kept = 0;
    }

    /// The cards, cursor and history, for parking.
    #[must_use]
    pub fn park_state(&self) -> (Vec<Card>, usize, Vec<A>)
    where
        A: Clone,
    {
        (self.cards.clone(), self.index, self.history.clone())
    }

    /// Restores a parked stack, the cursor kept within the cards.
    pub fn restore(&mut self, cards: Vec<Card>, index: usize, history: Vec<A>) {
        self.index = index.min(cards.len());
        self.cards = cards;
        self.history = history;
        self.entering = None;
        self.kept = 0;
    }

    /// Drops the card under the cursor so the next one slides in. Does nothing past the end.
    pub fn remove_current(&mut self) {
        if self.index < self.cards.len() {
            self.cards.remove(self.index);
        }
    }

    /// Puts a card back under the cursor, the undo of [`CardStack::remove_current`].
    pub fn insert_current(&mut self, card: Card) {
        let at = self.index.min(self.cards.len());
        self.cards.insert(at, card);
    }

    /// Swaps the card under the cursor for another, such as a chosen printing. Does nothing past the end.
    pub fn replace_current(&mut self, card: Card) {
        if let Some(slot) = self.cards.get_mut(self.index) {
            *slot = card;
        }
    }

    /// Moves a linear stack's cursor forward; it may land one past the end. Returns false when it already was there.
    pub fn advance(&mut self) -> bool {
        let next = self.index.saturating_add(1);
        if next <= self.cards.len() {
            self.index = next;
            true
        } else {
            false
        }
    }

    /// Moves a cycling stack's cursor forward, wrapping to the first card.
    pub fn advance_wrapping(&mut self) {
        if let Some(next) = self.index.saturating_add(1).checked_rem(self.cards.len()) {
            self.index = next;
        }
    }

    /// Moves a cycling stack's cursor back, wrapping to the last card. Returns false when the stack is empty.
    pub fn retreat_wrapping(&mut self) -> bool {
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

    /// Records a committed swipe for undo.
    pub fn record(&mut self, action: A) {
        self.history.push(action);
    }

    /// The newest swipe, taken off the history.
    pub fn pop_action(&mut self) -> Option<A> {
        self.history.pop()
    }

    /// Reverses a [`CardStack::step_back`] after its undo failed: records the action again and moves the cursor forward.
    pub fn unwind_undo(&mut self, action: A) {
        self.history.push(action);
        self.index = self.index.saturating_add(1);
        self.entering = None;
    }

    /// Marks the next card as entering from `direction`, for the client's motion.
    pub fn prime_entering(&mut self, direction: Direction) {
        self.entering = Some(direction);
    }

    /// Clears the entering mark.
    pub fn cancel_entering(&mut self) {
        self.entering = None;
    }
}

impl<A: StackAction> CardStack<A> {
    /// Steps a linear stack's cursor back onto the card `action` swiped and marks it as coming back from the side it left. Returns false at the first card.
    pub fn step_back(&mut self, action: &A) -> bool {
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

    #[test]
    fn the_ghost_holds_until_the_top_card_shows_and_has_had_time_to_paint() {
        let (top, under) = (Uuid::from_u128(1), Uuid::from_u128(2));
        let mut shown = HashSet::new();
        assert!(ghost_held(Some(top), &shown, None), "nothing shown yet");
        shown.insert(under);
        assert!(
            ghost_held(Some(top), &shown, None),
            "a card under the top showing does not release it"
        );
        shown.insert(top);
        assert!(
            ghost_held(Some(top), &shown, Some(top)),
            "came in while on top: held while it paints"
        );
        assert!(!ghost_held(Some(top), &shown, None), "shown and painted");
        assert!(!ghost_held(None, &shown, None), "no top, no ghost");
    }

    #[test]
    fn a_card_that_loaded_under_the_top_shows_at_once_when_it_comes_up() {
        // Every swipe makes the card under the top the new top. It loaded
        // there, so the stack must not hold its ghost (and hide the card
        // flying off) while it waits again.
        let (gone, next) = (Uuid::from_u128(1), Uuid::from_u128(2));
        let shown: HashSet<Uuid> = [gone, next].into_iter().collect();
        assert!(
            !ghost_held(Some(next), &shown, None),
            "the new top shows at once"
        );
        assert!(
            !ghost_held(Some(next), &shown, Some(gone)),
            "the old top still painting does not hold the new one"
        );
    }

    fn stack(names: &[&str]) -> CardStack<AddAction> {
        let mut stack = CardStack::default();
        stack.replace(names.iter().map(|n| card(n)).collect());
        stack
    }

    fn name(card: Option<&Card>) -> Option<&str> {
        card.map(|c| c.scryfall_data.name.as_str())
    }

    #[test]
    fn replacing_ahead_keeps_the_swiped_cards_for_undo() {
        let mut stack = stack(&["A", "B"]);
        assert!(stack.advance());
        stack.record(AddAction::Skip);
        let a = stack.cards()[0].clone();
        stack.replace_ahead(vec![card("C"), a]);
        assert_eq!(name(stack.current()), Some("C"));
        assert_eq!(stack.len(), 2, "B dropped, the swiped A not dealt twice");
        assert!(stack.has_served());
        let action = stack.pop_action().unwrap();
        assert!(stack.step_back(&action));
        assert_eq!(name(stack.current()), Some("A"));
    }

    #[test]
    fn clearing_ahead_leaves_nothing_served() {
        let mut stack = stack(&["A", "B"]);
        assert!(stack.advance());
        stack.clear_ahead();
        assert!(!stack.has_served());
        assert!(!stack.is_empty());
        stack.reset();
        stack.replace(vec![card("C")]);
        assert!(stack.has_served());
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
    fn the_card_under_the_top_wraps_only_on_a_cycling_stack() {
        let mut stack = stack(&["A", "B"]);
        assert_eq!(name(stack.next()), Some("B"));
        assert_eq!(name(stack.next_wrapping()), Some("B"));
        stack.advance();
        assert_eq!(stack.next(), None);
        assert_eq!(name(stack.next_wrapping()), Some("A"));
        let lone = self::stack(&["A"]);
        assert_eq!(
            lone.next_wrapping(),
            None,
            "a lone card is not under itself"
        );
    }

    #[test]
    fn each_action_names_its_exit_side() {
        assert_eq!(AddAction::Maybe.exited(), Direction::Up);
        assert_eq!(MaybeboardAction::Skip.exited(), Direction::Left);
        assert_eq!(RemoveAction::Keep.exited(), Direction::Left);
    }

    #[test]
    fn retreat_after_removing_the_last_card_lands_in_range() {
        let mut stack = stack(&["a", "b", "c"]);
        stack.restore(stack.cards().to_vec(), 2, Vec::new());
        stack.remove_current();
        assert_eq!(stack.index(), 2, "one past the end after the removal");
        assert!(stack.retreat_wrapping());
        assert_eq!(name(stack.current()), Some("b"));
        let mut empty: CardStack<AddAction> = CardStack::default();
        assert!(!empty.retreat_wrapping());
    }
}
