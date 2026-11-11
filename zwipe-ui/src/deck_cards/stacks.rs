//! The deck cards view as stacks: one pile of overlapping cards per group,
//! each card tucked under the one above it so only its title strip shows,
//! the last card in full. What the web and the phones share: the List |
//! Stacks choice, which cards tuck, and the reveal and open state behind
//! hover, taps and the details drawer or sheet. Clients draw the piles and
//! wire their own gestures to [`Reveal::apply`].

use uuid::Uuid;

/// How the deck page lists its cards.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
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
/// above it, showing only its title strip over that card. The first card
/// sits on top of the pile; every later one tucks, except the card right
/// after `revealed`, which drops below the revealed card so it shows in
/// full.
#[must_use]
pub fn tucked(count: usize, revealed: Option<usize>) -> Vec<bool> {
    (0..count)
        .map(|i| i > 0 && revealed.is_none_or(|r| r.checked_add(1) != Some(i)))
        .collect()
}

/// One step of the stacks' reveal and open state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    /// The pointer is over a card (web): it reveals in place.
    Hover(Uuid),
    /// The pointer left a card (web): it tucks back unless it is open.
    Leave(Uuid),
    /// A click on a card (web): its details open, in place of any open one.
    Click(Uuid),
    /// A tap on a card (phones): the first reveals it, a second on the same
    /// card opens its details.
    Tap(Uuid),
    /// The details closed (their ×, Escape, the backdrop, the back gesture).
    Close,
}

/// Which card shows in full and which card's details are open. The details
/// never follow the reveal: hovering across a stack while they are open
/// leaves the open card where it is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reveal {
    revealed: Option<Uuid>,
    open: Option<Uuid>,
}

impl Reveal {
    /// The card shown in full in its stack.
    #[must_use]
    pub const fn revealed(&self) -> Option<Uuid> {
        self.revealed
    }

    /// The card whose details are open.
    #[must_use]
    pub const fn open(&self) -> Option<Uuid> {
        self.open
    }

    /// Applies `gesture`.
    pub fn apply(&mut self, gesture: Gesture) {
        match gesture {
            Gesture::Hover(card) => self.revealed = Some(card),
            Gesture::Leave(card) => {
                if self.revealed == Some(card) {
                    self.revealed = None;
                }
            }
            Gesture::Click(card) => self.open = Some(card),
            Gesture::Tap(card) => {
                if self.revealed == Some(card) {
                    self.open = Some(card);
                } else {
                    self.revealed = Some(card);
                }
            }
            Gesture::Close => self.open = None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(tucked(0, None), Vec::<bool>::new());
        assert_eq!(tucked(1, None), [false]);
        assert_eq!(tucked(4, None), [false, true, true, true]);
    }

    #[test]
    fn the_card_after_the_revealed_one_drops_below_it() {
        assert_eq!(tucked(4, Some(1)), [false, true, false, true]);
        assert_eq!(tucked(4, Some(0)), [false, false, true, true]);
        assert_eq!(
            tucked(4, Some(3)),
            [false, true, true, true],
            "the last card shows in full anyway"
        );
        assert_eq!(
            tucked(4, Some(9)),
            [false, true, true, true],
            "a stale index changes nothing"
        );
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
