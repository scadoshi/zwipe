//! The browser history entries the open sheets and dialogs own: each overlay that opens adds an
//! entry at the screen's own URL, so the browser's back closes the top overlay by going back over
//! its entry, and a phone's back button, which skips entries a page adds without a tap, still
//! counts each one. An overlay closed another way (its ×, Esc, the backdrop) takes its entry back
//! out, and that move's pop is the page's own, so it changes nothing.
//!
//! This module only counts. [`leave_guard`](super::leave_guard) applies the moves to the browser
//! and asks it what each pop means.

/// The entries the open overlays own, and the pops the page started itself.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct OverlaySteps {
    /// Entries on top of the screen's own, one for each overlay that opened since it showed.
    owned: usize,
    /// Pops the page started itself to take entries back out, still to arrive.
    swallow: usize,
    /// Whether the next pop is the router's own back past every owned entry.
    passing: bool,
    /// Whether the last move was the browser's back closing the top overlay, which an overlay
    /// rising in its place, such as a stack's revealed card under its closed drawer, follows.
    after_pop: bool,
}

/// What the browser's history does to follow the open overlays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepMove {
    /// Adds this many entries at the screen's URL.
    Push(usize),
    /// Goes back over this many owned entries in one move.
    Back(usize),
    /// Closes this many top overlays: ones that rose as the browser's back closed the one above
    /// them. An entry added then has no tap behind it, which a phone's back skips, so the back
    /// closes them too.
    Close(usize),
}

impl StepMove {
    /// The browser script that makes the move; `None` for [`StepMove::Close`], which the overlay
    /// stack makes.
    pub(crate) fn script(self) -> Option<String> {
        Some(match self {
            Self::Push(n) => format!(
                "for (let i = 0; i < {n}; i++) history.pushState(history.state, '', location.href);"
            ),
            Self::Back(n) => format!("history.go(-{n});"),
            Self::Close(_) => return None,
        })
    }
}

/// What a pop of the browser's history means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopMeaning {
    /// The page's own move taking entries back out: nothing happens.
    Own,
    /// The browser went back over an overlay's entry: the top overlay closes.
    CloseTop,
    /// A move between routes, for the router.
    Route,
}

impl OverlaySteps {
    /// Follows the stack at `depth` open overlays: an entry for each one that opened, and one
    /// move back over the entries of those that closed some other way.
    pub(crate) fn follow(&mut self, depth: usize) -> Option<StepMove> {
        let after_pop = std::mem::take(&mut self.after_pop);
        if depth > self.owned && after_pop {
            return Some(StepMove::Close(depth.saturating_sub(self.owned)));
        }
        if depth > self.owned {
            let added = depth.saturating_sub(self.owned);
            self.owned = depth;
            return Some(StepMove::Push(added));
        }
        if depth < self.owned {
            let dropped = self.owned.saturating_sub(depth);
            self.owned = depth;
            self.swallow = self.swallow.saturating_add(1);
            return Some(StepMove::Back(dropped));
        }
        None
    }

    /// Reads a pop of the browser's history.
    pub(crate) fn pop(&mut self) -> PopMeaning {
        if self.passing {
            self.passing = false;
            return PopMeaning::Route;
        }
        if self.swallow > 0 {
            self.swallow = self.swallow.saturating_sub(1);
            return PopMeaning::Own;
        }
        if self.owned > 0 {
            self.owned = self.owned.saturating_sub(1);
            self.after_pop = true;
            return PopMeaning::CloseTop;
        }
        PopMeaning::Route
    }

    /// The router's back with overlays open: how many owned entries its move goes back over on
    /// top of the route's own, and the pop that move brings is the router's.
    pub(crate) fn leave(&mut self) -> usize {
        let owned = self.owned;
        self.owned = 0;
        if owned > 0 {
            self.passing = true;
        }
        owned
    }

    /// The screen moved to another route: the entries the overlays owned stay behind in the
    /// browser's history as plain entries of the screen left.
    pub(crate) fn route_moved(&mut self) {
        self.owned = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_opened_overlay_adds_an_entry_and_back_closes_the_top_one() {
        let mut steps = OverlaySteps::default();
        assert_eq!(steps.follow(1), Some(StepMove::Push(1)), "the drawer opens");
        assert_eq!(
            steps.follow(2),
            Some(StepMove::Push(1)),
            "Printings over it"
        );
        assert_eq!(steps.pop(), PopMeaning::CloseTop, "back closes Printings");
        assert_eq!(steps.follow(1), None, "its entry is already gone");
        assert_eq!(steps.pop(), PopMeaning::CloseTop, "back closes the drawer");
        assert_eq!(steps.follow(0), None);
        assert_eq!(
            steps.pop(),
            PopMeaning::Route,
            "then back leaves the screen"
        );
    }

    #[test]
    fn an_overlay_closed_another_way_takes_its_entry_out_and_its_pop_does_nothing() {
        let mut steps = OverlaySteps::default();
        steps.follow(2);
        assert_eq!(
            steps.follow(1),
            Some(StepMove::Back(1)),
            "Esc closes the top one"
        );
        assert_eq!(steps.pop(), PopMeaning::Own, "the page's own move back");
        assert_eq!(
            steps.pop(),
            PopMeaning::CloseTop,
            "back still closes the other"
        );
        assert_eq!(steps.follow(0), None);
    }

    #[test]
    fn overlays_closing_together_go_back_in_one_move() {
        let mut steps = OverlaySteps::default();
        steps.follow(3);
        assert_eq!(steps.follow(0), Some(StepMove::Back(3)));
        assert_eq!(steps.pop(), PopMeaning::Own, "one pop for the one move");
        assert_eq!(steps.pop(), PopMeaning::Route);
    }

    #[test]
    fn a_route_move_leaves_the_entries_behind_without_going_back() {
        let mut steps = OverlaySteps::default();
        steps.follow(1);
        steps.route_moved();
        assert_eq!(
            steps.follow(0),
            None,
            "the closing sheet does not undo the move"
        );
        assert_eq!(steps.pop(), PopMeaning::Route);
    }

    #[test]
    fn the_routers_back_goes_past_the_owned_entries_and_its_pop_is_the_routers() {
        let mut steps = OverlaySteps::default();
        steps.follow(2);
        assert_eq!(steps.leave(), 2, "two entries on top of the route's");
        assert_eq!(steps.pop(), PopMeaning::Route);
        assert_eq!(
            steps.follow(0),
            None,
            "the overlays closing do not move again"
        );
        assert_eq!(steps.leave(), 0, "nothing owned, a plain back");
        assert_eq!(steps.pop(), PopMeaning::Route);
    }

    #[test]
    fn an_overlay_rising_as_the_back_closes_the_one_above_it_closes_too() {
        let mut steps = OverlaySteps::default();
        steps.follow(1);
        assert_eq!(
            steps.follow(2),
            Some(StepMove::Push(1)),
            "the cards layer, the drawer"
        );
        assert_eq!(steps.follow(3), Some(StepMove::Push(1)), "Printings");
        assert_eq!(steps.pop(), PopMeaning::CloseTop, "back closes Printings");
        assert_eq!(steps.follow(2), None);
        assert_eq!(steps.pop(), PopMeaning::CloseTop, "back closes the drawer");
        assert_eq!(
            steps.follow(2),
            Some(StepMove::Close(1)),
            "the revealed card under it settles with no entry of its own"
        );
        assert_eq!(steps.follow(1), None);
        assert_eq!(
            steps.pop(),
            PopMeaning::CloseTop,
            "back closes the cards layer"
        );
    }

    #[test]
    fn an_overlay_opened_by_a_tap_after_a_back_still_gets_its_entry() {
        let mut steps = OverlaySteps::default();
        steps.follow(1);
        steps.pop();
        assert_eq!(steps.follow(0), None);
        assert_eq!(steps.follow(1), Some(StepMove::Push(1)));
    }

    #[test]
    fn the_scripts_add_entries_at_the_same_url_and_go_back_in_one_move() {
        let push = StepMove::Push(2).script().unwrap_or_default();
        assert!(push.contains("history.pushState(history.state, '', location.href)"));
        assert!(push.contains("i < 2"));
        assert_eq!(
            StepMove::Back(3).script().as_deref(),
            Some("history.go(-3);")
        );
        assert_eq!(StepMove::Close(1).script(), None);
    }
}
