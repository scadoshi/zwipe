//! The screens this tab has visited inside the app, so the header's back
//! arrow can tell whether an earlier screen of the app sits behind this one.
//!
//! The web's history always reports that it can go back, even on a tab
//! opened straight onto a deep link, where going back leaves the app. The
//! trail keeps the app's own entries and where the tab sits among them:
//! the router's moves update it as they land, and the browser's back and
//! forward are read from the route they land on.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use dioxus::history::History;
use dioxus::prelude::*;

/// The app's entries in this tab and the one on screen.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Trail {
    /// The routes, oldest first.
    entries: Vec<String>,
    /// The index of the route on screen.
    at: usize,
}

impl Trail {
    /// A trail holding only `route`, the screen the tab opened on.
    fn new(route: String) -> Self {
        Self {
            entries: vec![route],
            at: 0,
        }
    }

    /// The route on screen.
    fn current(&self) -> Option<&str> {
        self.entries.get(self.at).map(String::as_str)
    }

    /// The app's route before the one on screen, if the tab has one.
    fn previous(&self) -> Option<&str> {
        let before = self.at.checked_sub(1)?;
        self.entries.get(before).map(String::as_str)
    }

    /// Whether an earlier screen of the app sits behind this one.
    fn can_go_back(&self) -> bool {
        self.previous().is_some()
    }

    /// `route` opened on top of the one on screen, dropping any ahead of it.
    /// The route already on screen adds nothing, as the history skips it.
    fn push(&mut self, route: String) {
        if self.current() == Some(route.as_str()) {
            return;
        }
        let next = self.at.saturating_add(1);
        self.entries.truncate(next);
        self.entries.push(route);
        self.at = next;
    }

    /// `route` in place of the one on screen.
    fn replace(&mut self, route: String) {
        if let Some(entry) = self.entries.get_mut(self.at) {
            *entry = route;
        }
    }

    /// The browser moved through its history and landed on `route`: one step
    /// back or forward when it is the neighbor, nothing when it is the route
    /// already on screen (a move the leave guard undid), and a fresh trail
    /// otherwise, since a longer jump leaves the app's place unknown.
    fn popped(&mut self, route: &str) {
        if self.current() == Some(route) {
            return;
        }
        if let Some(before) = self.at.checked_sub(1)
            && self.entries.get(before).map(String::as_str) == Some(route)
        {
            self.at = before;
            return;
        }
        let after = self.at.saturating_add(1);
        if self.entries.get(after).map(String::as_str) == Some(route) {
            self.at = after;
            return;
        }
        *self = Self::new(route.to_owned());
    }
}

thread_local! {
    /// The tab's trail. The router's updater runs outside any component and
    /// must be `Send`, so the trail lives here rather than in a signal.
    static TRAIL: RefCell<Option<Trail>> = const { RefCell::new(None) };
    /// The history under [`TrailHistory`], which the updater reads the
    /// landed route from.
    static INNER: RefCell<Option<Rc<dyn History>>> = const { RefCell::new(None) };
}

/// The route the history under the trail shows, once one is set.
fn landed_route() -> Option<String> {
    INNER.with(|i| i.borrow().as_ref().map(|inner| inner.current_route()))
}

/// Runs `f` on the tab's trail, once one is set.
fn with_trail(f: impl FnOnce(&mut Trail)) {
    TRAIL.with(|t| {
        if let Some(trail) = t.borrow_mut().as_mut() {
            f(trail);
        }
    });
}

/// A history that keeps the tab's trail as the router moves, so
/// `can_go_back` answers for the app's own entries rather than the tab's.
pub struct TrailHistory {
    inner: Rc<dyn History>,
}

impl std::fmt::Debug for TrailHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrailHistory").finish_non_exhaustive()
    }
}

impl TrailHistory {
    /// Keeps a trail over `inner`, starting from its route on screen.
    pub fn new(inner: Rc<dyn History>) -> Self {
        let start = Trail::new(inner.current_route());
        TRAIL.with(|t| *t.borrow_mut() = Some(start));
        INNER.with(|i| *i.borrow_mut() = Some(Rc::clone(&inner)));
        Self { inner }
    }

    /// Provides the trail over the history already provided (the leave
    /// guard's) to the calling component and everything under it. Call it
    /// once above the router, after the guard.
    pub fn provide() {
        let tracked: Rc<dyn History> = Rc::new(Self::new(dioxus::history::history()));
        provide_context(tracked);
    }
}

impl History for TrailHistory {
    fn current_route(&self) -> String {
        self.inner.current_route()
    }

    fn current_prefix(&self) -> Option<String> {
        self.inner.current_prefix()
    }

    fn can_go_back(&self) -> bool {
        TRAIL.with(|t| t.borrow().as_ref().is_some_and(Trail::can_go_back))
    }

    fn go_back(&self) {
        self.inner.go_back();
    }

    fn can_go_forward(&self) -> bool {
        self.inner.can_go_forward()
    }

    fn go_forward(&self) {
        self.inner.go_forward();
    }

    // The guard may hold a move for its dialog, so the trail records only a
    // move that landed.
    fn push(&self, route: String) {
        self.inner.push(route.clone());
        if self.inner.current_route() == route {
            with_trail(|trail| trail.push(route));
        }
    }

    fn replace(&self, path: String) {
        self.inner.replace(path.clone());
        if self.inner.current_route() == path {
            with_trail(|trail| trail.replace(path));
        }
    }

    fn external(&self, url: String) -> bool {
        self.inner.external(url)
    }

    fn updater(&self, callback: Arc<dyn Fn() + Send + Sync>) {
        self.inner.updater(Arc::new(move || {
            if let Some(landed) = landed_route() {
                with_trail(|trail| trail.popped(&landed));
            }
            callback();
        }));
    }

    fn include_prevent_default(&self) -> bool {
        self.inner.include_prevent_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(route: &str) -> Trail {
        Trail::new(route.to_owned())
    }

    #[test]
    fn a_fresh_tab_has_nothing_behind_it() {
        assert!(!at("/decks/new").can_go_back());
    }

    #[test]
    fn a_push_puts_the_screen_before_behind_it() {
        let mut trail = at("/playstyle");
        trail.push("/decks/new".to_owned());
        assert!(trail.can_go_back());
        assert_eq!(trail.previous(), Some("/playstyle"));
    }

    #[test]
    fn a_replace_swaps_the_screen_without_adding_one() {
        let mut trail = at("/playstyle");
        trail.push("/decks/new".to_owned());
        trail.replace("/decks/42".to_owned());
        assert_eq!(trail.previous(), Some("/playstyle"));
        trail.popped("/playstyle");
        assert!(!trail.can_go_back(), "the spent form is gone");
    }

    #[test]
    fn going_back_and_forward_moves_along_the_entries() {
        let mut trail = at("/");
        trail.push("/playstyle".to_owned());
        trail.push("/decks/new".to_owned());
        trail.popped("/playstyle");
        assert_eq!(trail.previous(), Some("/"));
        trail.popped("/decks/new");
        assert_eq!(trail.previous(), Some("/playstyle"), "forward");
        trail.popped("/playstyle");
        trail.popped("/");
        assert!(!trail.can_go_back());
    }

    #[test]
    fn a_push_after_going_back_drops_the_screens_ahead() {
        let mut trail = at("/");
        trail.push("/playstyle".to_owned());
        trail.popped("/");
        trail.push("/decks".to_owned());
        trail.popped("/");
        trail.popped("/playstyle");
        assert_eq!(
            trail.previous(),
            None,
            "an unknown landing starts over rather than guess"
        );
    }

    #[test]
    fn a_pop_that_lands_where_it_was_changes_nothing() {
        let mut trail = at("/");
        trail.push("/decks/new".to_owned());
        trail.popped("/decks/new");
        assert_eq!(
            trail.previous(),
            Some("/"),
            "an undone back keeps its place"
        );
    }

    #[test]
    fn pushing_the_screen_on_show_adds_nothing() {
        let mut trail = at("/decks");
        trail.push("/decks".to_owned());
        assert!(!trail.can_go_back());
    }
}
