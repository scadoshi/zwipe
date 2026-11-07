//! Asks before a screen with unsaved changes is left: the app's copy of the
//! web's guard (zwiper-web `chrome/leave_guard.rs`), with zwipe-ui's copy.
//! A screen arms it while it holds changes (`LeaveGuard::set`); while it
//! is armed, every router move waits on `LeaveDialog`: the header's back
//! arrow, the menu's links, any in-app link, a pushed route, and the OS back
//! (the iOS edge swipe, Android's back), which reaches the router once no
//! overlay is left to close.
//!
//! Stay drops the move and leaves every layer on the screen as it was.
//! Leave disarms the guard and makes the move.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

use dioxus::history::History;
use dioxus::prelude::*;

use crate::inbound::router::Router;

/// A move the guard held back.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Held {
    /// A new route.
    Push(String),
    /// A route in place of this one.
    Replace(String),
    /// The router's back.
    Back,
    /// The router's forward.
    Forward,
    /// A page outside the app.
    External(String),
}

/// What the dialog says, from the screen that armed the guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveCopy {
    /// The title.
    pub title: &'static str,
    /// The body.
    pub body: &'static str,
    /// The button that stays.
    pub stay: &'static str,
    /// The button that leaves.
    pub leave: &'static str,
}

/// The guard, one per app. The history the router calls runs outside any
/// component, so the state lives here.
#[derive(Default)]
struct Guard {
    /// The armed screen's copy; `None` while disarmed.
    copy: Cell<Option<LeaveCopy>>,
    /// The move waiting on the dialog.
    held: RefCell<Option<Held>>,
    /// The dialog's open state, from the mounted [`LeaveDialog`].
    asking: Cell<Option<Signal<bool>>>,
}

thread_local! {
    static GUARD: Guard = Guard::default();
}

/// Holds `held` and opens the dialog while the guard is armed. Returns
/// whether it held it.
fn hold(held: Held) -> bool {
    GUARD.with(|g| {
        if g.copy.get().is_none() {
            return false;
        }
        *g.held.borrow_mut() = Some(held);
        if let Some(mut asking) = g.asking.get() {
            asking.set(true);
        }
        true
    })
}

/// Arms the guard with `copy`, or disarms it with `None`.
fn arm(copy: Option<LeaveCopy>) {
    GUARD.with(|g| g.copy.set(copy));
}

/// The app's in-memory history with the guard in front: while a screen is
/// armed, each move the router makes is held for the dialog instead.
pub struct GuardedHistory {
    inner: Rc<dyn History>,
}

impl std::fmt::Debug for GuardedHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GuardedHistory").finish_non_exhaustive()
    }
}

impl GuardedHistory {
    /// Provides the guard in front of the platform's history to the calling
    /// component and everything under it. Call it once above the router.
    pub fn provide() {
        let guarded: Rc<dyn History> = Rc::new(Self {
            inner: dioxus::history::history(),
        });
        provide_context(guarded);
    }
}

impl History for GuardedHistory {
    fn current_route(&self) -> String {
        self.inner.current_route()
    }

    fn current_prefix(&self) -> Option<String> {
        self.inner.current_prefix()
    }

    fn can_go_back(&self) -> bool {
        self.inner.can_go_back()
    }

    fn go_back(&self) {
        if !hold(Held::Back) {
            self.inner.go_back();
        }
    }

    fn can_go_forward(&self) -> bool {
        self.inner.can_go_forward()
    }

    fn go_forward(&self) {
        if !hold(Held::Forward) {
            self.inner.go_forward();
        }
    }

    fn push(&self, route: String) {
        if route == self.inner.current_route() || !hold(Held::Push(route.clone())) {
            self.inner.push(route);
        }
    }

    fn replace(&self, path: String) {
        if !hold(Held::Replace(path.clone())) {
            self.inner.replace(path);
        }
    }

    fn external(&self, url: String) -> bool {
        hold(Held::External(url.clone())) || self.inner.external(url)
    }

    fn updater(&self, callback: Arc<dyn Fn() + Send + Sync>) {
        self.inner.updater(callback);
    }

    fn include_prevent_default(&self) -> bool {
        self.inner.include_prevent_default()
    }
}

/// The guard as a screen holds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveGuard {
    copy: LeaveCopy,
}

impl LeaveGuard {
    /// Arms the guard when `changed`, and disarms it otherwise. A screen
    /// calls it after every change to its model and before it performs the
    /// moves that change asks for, so a save's own move goes through.
    pub fn set(self, changed: bool) {
        arm(changed.then_some(self.copy));
    }
}

/// The guard for a screen that says `copy`, disarmed when the screen goes.
pub fn use_leave_guard(copy: LeaveCopy) -> LeaveGuard {
    use_drop(|| arm(None));
    LeaveGuard { copy }
}

/// The dialog the guard asks through, over every layer a screen can have
/// open (the pickers, the sheets, the swipe select), with the kit's alert
/// dialog's look. The OS back on it stays. Mount it once inside the router.
#[component]
pub fn LeaveDialog() -> Element {
    let navigator = use_navigator();
    let mut asking = use_hook(|| {
        let asking = Signal::new(false);
        GUARD.with(|g| g.asking.set(Some(asking)));
        asking
    });
    use_drop(|| GUARD.with(|g| g.asking.set(None)));

    let stay = use_callback(move |()| {
        GUARD.with(|g| g.held.borrow_mut().take());
        asking.set(false);
    });
    components::use_overlay_back_action(asking.into(), stay);
    let mut leave = move || {
        let held = GUARD.with(|g| g.held.borrow_mut().take());
        arm(None);
        asking.set(false);
        let to_route = |path: &str| Router::from_str(path).unwrap_or(Router::Home {});
        match held {
            Some(Held::Push(path)) => {
                navigator.push(to_route(&path));
            }
            Some(Held::Replace(path)) => {
                navigator.replace(to_route(&path));
            }
            Some(Held::Back) => navigator.go_back(),
            Some(Held::Forward) => navigator.go_forward(),
            Some(Held::External(url)) => {
                navigator.push(NavigationTarget::<Router>::External(url));
            }
            None => {}
        }
    };

    let copy = GUARD.with(|g| g.copy.get());
    rsx! {
        if let (true, Some(copy)) = (asking(), copy) {
            div {
                class: "alert-dialog-overlay",
                style: "z-index: 1002;",
                "data-state": "open",
                onclick: move |_| stay.call(()),
            }
            div {
                class: "alert-dialog",
                style: "z-index: 1003;",
                role: "alertdialog",
                aria_modal: "true",
                "aria-labelledby": "leave-title",
                "aria-describedby": "leave-body",
                "data-state": "open",
                h2 { id: "leave-title", class: "alert-dialog-title", "{copy.title}" }
                hr { class: "alert-dialog-rule" }
                div { id: "leave-body", class: "alert-dialog-description", "{copy.body}" }
                hr { class: "alert-dialog-rule" }
                div { class: "alert-dialog-actions",
                    button {
                        class: "alert-dialog-cancel",
                        r#type: "button",
                        onclick: move |_| stay.call(()),
                        "{copy.stay}"
                    }
                    button {
                        class: "alert-dialog-action-danger",
                        r#type: "button",
                        onclick: move |_| leave(),
                        "{copy.leave}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COPY: LeaveCopy = LeaveCopy {
        title: "t",
        body: "b",
        stay: "s",
        leave: "l",
    };

    #[test]
    fn a_disarmed_guard_holds_nothing() {
        arm(None);
        assert!(!hold(Held::Back));
        assert!(!hold(Held::Push("/decks".to_string())));
    }

    #[test]
    fn an_armed_guard_holds_the_move_until_disarmed() {
        arm(Some(COPY));
        assert!(hold(Held::Back), "back waits on the dialog");
        assert_eq!(GUARD.with(|g| g.held.borrow().clone()), Some(Held::Back));
        assert!(hold(Held::Push("/decks".to_string())), "a link waits too");
        assert_eq!(
            GUARD.with(|g| g.held.borrow().clone()),
            Some(Held::Push("/decks".to_string())),
            "the latest move waits"
        );
        let guard = LeaveGuard { copy: COPY };
        guard.set(false);
        assert!(!hold(Held::Back), "a saved form lets every move through");
        guard.set(true);
        assert!(hold(Held::Replace("/".to_string())));
        arm(None);
    }
}
