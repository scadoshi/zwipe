//! The deck cards view as stacks: one pile of overlapping cards per group,
//! each card tucked under the one above it so only its title strip shows,
//! the last card in full. What the web and the phones share: the List |
//! Stacks choice, which cards tuck, and the reveal and open state behind
//! hover, taps and the details drawer or sheet. Clients draw the piles and
//! wire their own gestures to [`Reveal::apply`].

use std::time::Duration;

use uuid::Uuid;

/// How the deck page lists its cards.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CardsView {
    /// The grouped rows, the default.
    #[default]
    List,
    /// One stack of overlapping cards per group.
    Stacks,
}

impl CardsView {
    /// The choice as stored, so the page opens on it next time.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Stacks => "stacks",
        }
    }

    /// The choice stored under `key`, `None` for anything else.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        [Self::List, Self::Stacks]
            .into_iter()
            .find(|view| view.key() == key)
    }

    /// The switch's label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::List => "List",
            Self::Stacks => "Stacks",
        }
    }
}

/// The switch's label chip.
pub const VIEW_LABEL: &str = "View";

/// For each of a stack's `count` cards, whether it tucks under the card
/// above it, showing only its title strip over that card: every card after
/// the first. This is the stack's layout, which a reveal never changes; the
/// reveal moves cards with [`shifted`].
#[must_use]
pub fn tucked(count: usize) -> Vec<bool> {
    (0..count).map(|i| i > 0).collect()
}

/// For each of a stack's `count` cards, whether it shows in full: the
/// `revealed` one and the last, which nothing covers. Only a card shown in
/// full wears the open card's outline.
#[must_use]
pub fn exposed(count: usize, revealed: Option<usize>) -> Vec<bool> {
    let last = count.checked_sub(1);
    (0..count)
        .map(|i| Some(i) == revealed || Some(i) == last)
        .collect()
}

/// For each of a stack's `count` cards, whether it moves down to make room
/// for `revealed`: every card after the revealed one does, by the height its
/// tuck hid, so the revealed card shows whole. The move is a transform, which
/// leaves the stack's layout alone.
#[must_use]
pub fn shifted(count: usize, revealed: Option<usize>) -> Vec<bool> {
    (0..count)
        .map(|i| revealed.is_some_and(|r| i > r))
        .collect()
}

/// One step of the stacks' reveal and open state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gesture {
    /// The pointer is over a card (web): it reveals in place.
    Hover(Uuid),
    /// The pointer left a card (web): the reveal goes back to the open card,
    /// or to none.
    Leave(Uuid),
    /// A click on a card (web): its details open, in place of any open one;
    /// a click on the open card closes them.
    Click(Uuid),
    /// A tap on a card (phones): the first reveals it, a second on the same
    /// card opens its details, and one more closes them.
    Tap(Uuid),
    /// The details closed (their ×, Escape, the backdrop, the back gesture).
    Close,
}

/// The pause between one card's deal and the next in a stack, top to bottom.
pub const DEAL_STAGGER: Duration = Duration::from_millis(18);

/// The card after which the stagger stops growing, so a long stack still deals at once.
pub const DEAL_STAGGER_CAP: u32 = 16;

/// How long one card takes to rise into its slot.
pub const DEAL_DURATION: Duration = Duration::from_millis(380);

/// How long after the deal starts the card at `index` in its stack (`0` the top) starts rising:
/// the stagger per card above it, up to the cap.
#[must_use]
pub fn deal_delay(index: u32) -> Duration {
    DEAL_STAGGER.saturating_mul(index.min(DEAL_STAGGER_CAP))
}

/// When the deck page deals its stacks in: once on entering the Stacks view, as soon as the cards
/// are there, and never again until the page leaves Stacks and comes back. A rerender, a filter or
/// a regrouping deals nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DealCue {
    /// The view at the last call; `None` before the first.
    last: Option<CardsView>,
    /// Whether a deal waits for the cards.
    armed: bool,
}

impl DealCue {
    /// The page shows `view`, with its cards in when `loaded`: whether the stacks deal now.
    pub fn on(&mut self, view: CardsView, loaded: bool) -> bool {
        let stacks = view == CardsView::Stacks;
        if stacks && self.last != Some(CardsView::Stacks) {
            self.armed = true;
        }
        if !stacks {
            self.armed = false;
        }
        self.last = Some(view);
        let deal = self.armed && loaded;
        if deal {
            self.armed = false;
        }
        deal
    }
}

/// Which card shows in full and which card's details are open. The open
/// card is pinned: it stays revealed when the pointer leaves it, a card under
/// the pointer is revealed in its place (one at a time), and leaving every
/// card brings the reveal back to it. The details never follow the reveal;
/// only a click (or a second tap) moves them to another card, and a click
/// on the open card closes them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Reveal {
    /// The card under the pointer, or the last tapped.
    hovered: Option<Uuid>,
    /// The card whose details are open, the pin.
    open: Option<Uuid>,
    /// The last card whose details opened, kept through a close.
    drawn: Option<Uuid>,
}

impl Reveal {
    /// The card shown in full in the stack of `cards`: the one under the
    /// pointer when it is in this stack, else the open one when it is, so a
    /// hover in another stack leaves this one's open card revealed.
    #[must_use]
    pub fn revealed_in(&self, cards: &[Uuid]) -> Option<Uuid> {
        let here = |card: Option<Uuid>| card.filter(|c| cards.contains(c));
        here(self.hovered).or_else(|| here(self.open))
    }

    /// The card shown in full in its stack: the one under the pointer, else
    /// the open one.
    #[must_use]
    pub const fn revealed(&self) -> Option<Uuid> {
        match self.hovered {
            Some(card) => Some(card),
            None => self.open,
        }
    }

    /// The card whose details are open.
    #[must_use]
    pub const fn open(&self) -> Option<Uuid> {
        self.open
    }

    /// The card the details drawer shows: the open one, or after a close the
    /// last one opened, so the drawer eases out with its card still in it.
    #[must_use]
    pub const fn drawn(&self) -> Option<Uuid> {
        self.drawn
    }

    /// Opens `card`'s details, or closes them when they are already open.
    fn pin(&mut self, card: Uuid) {
        if self.open == Some(card) {
            self.open = None;
        } else {
            self.open = Some(card);
            self.drawn = Some(card);
        }
    }

    /// Applies `gesture`.
    pub fn apply(&mut self, gesture: Gesture) {
        match gesture {
            Gesture::Hover(card) => self.hovered = Some(card),
            Gesture::Leave(card) => {
                if self.hovered == Some(card) {
                    self.hovered = None;
                }
            }
            Gesture::Click(card) => self.pin(card),
            Gesture::Tap(card) => {
                if self.revealed() == Some(card) {
                    self.pin(card);
                } else {
                    self.hovered = Some(card);
                }
            }
            Gesture::Close => self.open = None,
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn entering_stacks_deals_once_the_cards_are_in() {
        let mut cue = DealCue::default();
        assert!(
            !cue.on(CardsView::Stacks, false),
            "the first load waits for its cards"
        );
        assert!(cue.on(CardsView::Stacks, true), "and deals when they land");
        assert!(
            !cue.on(CardsView::Stacks, true),
            "a rerender or a filter change deals nothing"
        );
        assert!(!cue.on(CardsView::List, true));
        assert!(
            cue.on(CardsView::Stacks, true),
            "switching back into Stacks deals again"
        );
        let mut list_first = DealCue::default();
        assert!(
            !list_first.on(CardsView::List, true),
            "the List never deals"
        );
        assert!(list_first.on(CardsView::Stacks, true));
    }

    #[test]
    fn a_whole_deal_is_brisk_even_for_a_long_stack() {
        let longest = DEAL_STAGGER
            .saturating_mul(DEAL_STAGGER_CAP)
            .saturating_add(DEAL_DURATION);
        assert!(
            longest < std::time::Duration::from_millis(1000),
            "a 30-card stack deals in {longest:?}"
        );
        assert_eq!(
            deal_delay(0),
            std::time::Duration::ZERO,
            "the top card enters first"
        );
        assert_eq!(
            deal_delay(30),
            deal_delay(DEAL_STAGGER_CAP),
            "the stagger stops at its cap"
        );
    }
    use super::*;

    #[test]
    fn only_the_revealed_card_and_the_last_show_in_full() {
        assert_eq!(exposed(4, None), [false, false, false, true]);
        assert_eq!(exposed(4, Some(1)), [false, true, false, true]);
        assert_eq!(exposed(4, Some(3)), [false, false, false, true]);
        assert_eq!(exposed(0, None), [false; 0]);
    }

    #[test]
    fn a_hover_in_another_stack_leaves_the_pinned_card_revealed() {
        let mut reveal = Reveal::default();
        let left = [id(1), id(2), id(3)];
        let right = [id(7), id(8)];
        reveal.apply(Gesture::Click(id(1)));
        reveal.apply(Gesture::Hover(id(7)));
        assert_eq!(
            reveal.revealed_in(&left),
            Some(id(1)),
            "the pinned stack holds its card"
        );
        assert_eq!(reveal.revealed_in(&right), Some(id(7)));
        reveal.apply(Gesture::Hover(id(3)));
        assert_eq!(
            reveal.revealed_in(&left),
            Some(id(3)),
            "a hover in its own stack still wins"
        );
        assert_eq!(reveal.revealed_in(&right), None);
    }

    #[test]
    fn the_cards_after_the_revealed_one_move_down() {
        assert_eq!(shifted(4, None), [false, false, false, false]);
        assert_eq!(shifted(4, Some(1)), [false, false, true, true]);
        assert_eq!(shifted(4, Some(0)), [false, true, true, true]);
        assert_eq!(
            shifted(4, Some(3)),
            [false, false, false, false],
            "nothing under the last card"
        );
        assert_eq!(
            shifted(4, Some(9)),
            [false, false, false, false],
            "a stale index moves nothing"
        );
    }

    #[test]
    fn hovering_over_a_pinned_card_reveals_the_hovered_one() {
        let mut reveal = Reveal::default();
        reveal.apply(Gesture::Click(id(1)));
        reveal.apply(Gesture::Leave(id(1)));
        assert_eq!(
            reveal.revealed(),
            Some(id(1)),
            "the clicked card stays revealed after the cursor leaves it"
        );
        reveal.apply(Gesture::Hover(id(2)));
        assert_eq!(
            reveal.revealed(),
            Some(id(2)),
            "only one card is revealed at a time"
        );
        assert_eq!(
            reveal.open(),
            Some(id(1)),
            "the drawer keeps the clicked card"
        );
    }

    #[test]
    fn leaving_every_card_returns_to_the_pinned_one() {
        let mut reveal = Reveal::default();
        reveal.apply(Gesture::Click(id(1)));
        reveal.apply(Gesture::Hover(id(2)));
        reveal.apply(Gesture::Leave(id(2)));
        assert_eq!(reveal.revealed(), Some(id(1)));
    }

    #[test]
    fn a_click_on_another_card_repins_it() {
        let mut reveal = Reveal::default();
        reveal.apply(Gesture::Click(id(1)));
        reveal.apply(Gesture::Hover(id(2)));
        reveal.apply(Gesture::Click(id(2)));
        reveal.apply(Gesture::Leave(id(2)));
        assert_eq!(
            (reveal.revealed(), reveal.open()),
            (Some(id(2)), Some(id(2)))
        );
    }

    #[test]
    fn the_drawer_keeps_its_last_card_through_a_close() {
        let mut reveal = Reveal::default();
        assert_eq!(reveal.drawn(), None, "nothing drawn before a click");
        reveal.apply(Gesture::Click(id(1)));
        reveal.apply(Gesture::Click(id(2)));
        assert_eq!(reveal.drawn(), Some(id(2)), "a second click swaps the card");
        reveal.apply(Gesture::Hover(id(3)));
        assert_eq!(reveal.drawn(), Some(id(2)), "hovering never swaps it");
        reveal.apply(Gesture::Close);
        assert_eq!(
            (reveal.open(), reveal.drawn()),
            (None, Some(id(2))),
            "the closing drawer still shows its card as it eases out"
        );
    }

    #[test]
    fn clicking_the_pinned_card_closes_it() {
        let mut reveal = Reveal::default();
        reveal.apply(Gesture::Click(id(1)));
        reveal.apply(Gesture::Click(id(1)));
        assert_eq!(reveal.open(), None, "a second click closes, as Escape does");
        assert_eq!(
            reveal.drawn(),
            Some(id(1)),
            "the drawer eases out with its card"
        );
        reveal.apply(Gesture::Click(id(1)));
        reveal.apply(Gesture::Click(id(2)));
        assert_eq!(reveal.open(), Some(id(2)), "another card still swaps");
    }

    #[test]
    fn tapping_the_pinned_card_closes_it() {
        let mut reveal = Reveal::default();
        reveal.apply(Gesture::Tap(id(1)));
        reveal.apply(Gesture::Tap(id(1)));
        assert_eq!(reveal.open(), Some(id(1)), "the second tap opens");
        reveal.apply(Gesture::Tap(id(1)));
        assert_eq!(reveal.open(), None, "a tap on the open card closes it");
    }

    #[test]
    fn closing_clears_the_pin() {
        let mut reveal = Reveal::default();
        reveal.apply(Gesture::Click(id(1)));
        reveal.apply(Gesture::Leave(id(1)));
        reveal.apply(Gesture::Close);
        assert_eq!(
            (reveal.revealed(), reveal.open()),
            (None, None),
            "with the drawer closed, leaving tucks back as before"
        );
        reveal.apply(Gesture::Click(id(1)));
        reveal.apply(Gesture::Hover(id(3)));
        reveal.apply(Gesture::Close);
        assert_eq!(
            reveal.revealed(),
            Some(id(3)),
            "closing under the cursor keeps the hovered card revealed"
        );
    }

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    #[test]
    fn the_view_round_trips_through_its_key() {
        for view in [CardsView::List, CardsView::Stacks] {
            assert_eq!(CardsView::from_key(view.key()), Some(view), "{view:?}");
        }
        assert_eq!(CardsView::from_key("grid"), None);
        assert_eq!(
            CardsView::default(),
            CardsView::List,
            "the list stays the default"
        );
    }

    #[test]
    fn every_card_after_the_first_tucks() {
        assert_eq!(tucked(0), Vec::<bool>::new());
        assert_eq!(tucked(1), [false]);
        assert_eq!(tucked(4), [false, true, true, true]);
    }

    #[test]
    fn hover_reveals_and_leaving_tucks_back() {
        let mut reveal = Reveal::default();
        reveal.apply(Gesture::Hover(id(1)));
        assert_eq!(reveal.revealed(), Some(id(1)));
        reveal.apply(Gesture::Hover(id(2)));
        assert_eq!(
            reveal.revealed(),
            Some(id(2)),
            "moving onto the next card reveals it"
        );
        reveal.apply(Gesture::Leave(id(1)));
        assert_eq!(
            reveal.revealed(),
            Some(id(2)),
            "a late leave of the old card changes nothing"
        );
        reveal.apply(Gesture::Leave(id(2)));
        assert_eq!(reveal.revealed(), None);
        assert_eq!(reveal.open(), None, "hover never opens the details");
    }

    #[test]
    fn a_click_opens_and_another_click_swaps() {
        let mut reveal = Reveal::default();
        reveal.apply(Gesture::Click(id(1)));
        assert_eq!(reveal.open(), Some(id(1)));
        reveal.apply(Gesture::Hover(id(3)));
        assert_eq!(reveal.open(), Some(id(1)), "the details don't follow hover");
        reveal.apply(Gesture::Click(id(2)));
        assert_eq!(
            reveal.open(),
            Some(id(2)),
            "a click on another card swaps it in"
        );
        reveal.apply(Gesture::Close);
        assert_eq!(reveal.open(), None);
    }

    #[test]
    fn a_first_tap_reveals_and_a_second_opens() {
        let mut reveal = Reveal::default();
        reveal.apply(Gesture::Tap(id(1)));
        assert_eq!((reveal.revealed(), reveal.open()), (Some(id(1)), None));
        reveal.apply(Gesture::Tap(id(2)));
        assert_eq!(
            (reveal.revealed(), reveal.open()),
            (Some(id(2)), None),
            "a tap on another card moves the reveal"
        );
        reveal.apply(Gesture::Tap(id(2)));
        assert_eq!(
            reveal.open(),
            Some(id(2)),
            "a second tap on the revealed card opens it"
        );
        reveal.apply(Gesture::Close);
        assert_eq!(
            (reveal.revealed(), reveal.open()),
            (Some(id(2)), None),
            "closing keeps the card revealed"
        );
    }
}
