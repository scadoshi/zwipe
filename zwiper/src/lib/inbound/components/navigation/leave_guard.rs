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
//!
//! It always lets go: only the mounted screen that owns it can arm it, so a
//! reply that lands after the screen has gone changes nothing; a move made
//! from any route but the one it armed on disarms it; and with no dialog
//! mounted to ask, a move goes through rather than vanishing.
//!
//! The state is one per process, not per thread: on Android the app renders
//! on one thread and handles taps on another, so a tap must find the guard
//! the screen armed while rendering.

use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

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
#[derive(Debug)]
struct Guard {
    /// The armed screen's copy; `None` while disarmed.
    copy: Option<LeaveCopy>,
    /// The route the guard armed on; a move from any other disarms it.
    route: Option<String>,
    /// The mounted screen that owns the guard, `0` for none.
    owner: u64,
    /// The last owner handed out.
    owners: u64,
    /// The move waiting on the dialog.
    held: Option<Held>,
    /// The dialog's open state, from the mounted [`LeaveDialog`].
    asking: Option<SyncSignal<bool>>,
}

static GUARD: Mutex<Guard> = Mutex::new(Guard {
    copy: None,
    route: None,
    owner: 0,
    owners: 0,
    held: None,
    asking: None,
});

/// Runs `f` on the guard. `None` when a panic elsewhere poisoned its lock,
/// and then every move goes through.
fn with_guard<R>(f: impl FnOnce(&mut Guard) -> R) -> Option<R> {
    let Ok(mut guard) = GUARD.lock() else {
        tracing::warn!("leave guard lock poisoned; moves go through");
        return None;
    };
    Some(f(&mut guard))
}

impl Guard {
    /// Holds `held` while the guard is armed on `here`, the route on view,
    /// and a dialog is mounted to ask, returning the dialog to open. A guard
    /// armed on another route disarms instead.
    fn hold(&mut self, held: Held, here: &str) -> Option<SyncSignal<bool>> {
        self.copy?;
        if self.route.as_deref().is_some_and(|armed| armed != here) {
            self.copy = None;
            self.route = None;
            return None;
        }
        let asking = self.asking?;
        self.held = Some(held);
        Some(asking)
    }

    /// Arms the guard with `copy`, recording `here` as the route it armed
    /// on, or disarms it with `None`.
    fn arm(&mut self, copy: Option<LeaveCopy>, here: Option<String>) {
        if copy.is_some() && self.copy.is_none() {
            self.route = here;
        }
        if copy.is_none() {
            self.route = None;
        }
        self.copy = copy;
    }
}

/// Holds `held` and opens the dialog while the guard is armed on `here` and
/// a dialog is mounted to ask. Returns whether it held it.
fn hold(held: Held, here: &str) -> bool {
    let Some(mut asking) = with_guard(|g| g.hold(held, here)).flatten() else {
        return false;
    };
    asking.set(true);
    true
}

/// Disarms the guard.
fn disarm() {
    with_guard(|g| g.arm(None, None));
}

/// Hands out a new owner and makes it the guard's.
fn take_ownership() -> u64 {
    with_guard(|g| {
        let owner = g.owners.wrapping_add(1).max(1);
        g.owners = owner;
        g.owner = owner;
        owner
    })
    .unwrap_or(0)
}

/// Arms or disarms the guard for `owner` on `here`, if it still owns it.
fn arm_for(owner: u64, copy: Option<LeaveCopy>, here: Option<String>) {
    with_guard(|g| {
        if g.owner == owner {
            g.arm(copy, here);
        }
    });
}

/// Lets `owner`'s hold go, disarming the guard, if it still owns it.
fn release(owner: u64) {
    with_guard(|g| {
        if g.owner == owner {
            g.owner = 0;
        }
        if g.owner == 0 {
            g.arm(None, None);
        }
    });
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
        let inner = dioxus::history::history();
        let guarded: Rc<dyn History> = Rc::new(Self { inner });
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
        if !hold(Held::Back, &self.inner.current_route()) {
            self.inner.go_back();
        }
    }

    fn can_go_forward(&self) -> bool {
        self.inner.can_go_forward()
    }

    fn go_forward(&self) {
        if !hold(Held::Forward, &self.inner.current_route()) {
            self.inner.go_forward();
        }
    }

    fn push(&self, route: String) {
        let here = self.inner.current_route();
        if route == here || !hold(Held::Push(route.clone()), &here) {
            self.inner.push(route);
        }
    }

    fn replace(&self, path: String) {
        if !hold(Held::Replace(path.clone()), &self.inner.current_route()) {
            self.inner.replace(path);
        }
    }

    fn external(&self, url: String) -> bool {
        hold(Held::External(url.clone()), &self.inner.current_route()) || self.inner.external(url)
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
    owner: u64,
}

impl LeaveGuard {
    /// Arms the guard when `changed`, and disarms it otherwise. A screen
    /// calls it after every change to its model and before it performs the
    /// moves that change asks for, so a save's own move goes through. A
    /// screen that no longer owns the guard changes nothing.
    pub fn set(self, changed: bool) {
        let here = try_consume_context::<Rc<dyn History>>().map(|history| history.current_route());
        arm_for(self.owner, changed.then_some(self.copy), here);
    }
}

/// The guard for a screen that says `copy`, owned while the screen is
/// mounted and disarmed when it goes.
pub fn use_leave_guard(copy: LeaveCopy) -> LeaveGuard {
    let owner = use_hook(take_ownership);
    use_drop(move || release(owner));
    LeaveGuard { copy, owner }
}

/// The dialog the guard asks through, over every layer a screen can have
/// open (the pickers, the sheets, the swipe select), with the kit's alert
/// dialog's look. The OS back on it stays. Mount it once inside the router.
#[component]
pub fn LeaveDialog() -> Element {
    let navigator = use_navigator();
    let mut asking = use_hook(|| {
        let asking = Signal::new_maybe_sync(false);
        with_guard(|g| g.asking = Some(asking));
        asking
    });
    use_drop(move || {
        with_guard(|g| {
            if g.asking == Some(asking) {
                g.asking = None;
            }
        });
    });
    let open = use_memo(move || *asking.read());

    let stay = use_callback(move |()| {
        with_guard(|g| g.held.take());
        asking.set(false);
    });
    components::use_overlay_back_action(open.into(), stay);
    let mut leave = move || {
        let held = with_guard(|g| g.held.take()).flatten();
        disarm();
        asking.set(false);
        let to_route = |path: &str| {
            Router::from_str(path).unwrap_or_else(|e| {
                tracing::warn!(%path, "held route unparsable; going home: {e}");
                Router::Home {}
            })
        };
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

    let copy = with_guard(|g| g.copy).flatten();
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
    use std::cell::RefCell;
    use std::sync::MutexGuard;
    use std::thread;

    use super::*;

    const COPY: LeaveCopy = LeaveCopy {
        title: "t",
        body: "b",
        stay: "s",
        leave: "l",
    };

    const EDIT: &str = "/decks/1/edit";

    /// The guard is one per process, so the tests take turns with it.
    static SERIAL: Mutex<()> = Mutex::new(());

    /// A fresh guard with no dialog, held for the test's length.
    fn fresh() -> MutexGuard<'static, ()> {
        let turn = SERIAL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        with_guard(|g| {
            *g = Guard {
                copy: None,
                route: None,
                owner: 0,
                owners: 0,
                held: None,
                asking: None,
            };
        });
        turn
    }

    /// Runs `test` in a runtime with a dialog mounted to ask, as the app has
    /// it.
    fn with_runtime(test: impl FnOnce()) {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        dom.in_runtime(|| {
            let asking = Signal::new_maybe_sync_in_scope(false, ScopeId::ROOT);
            with_guard(|g| g.asking = Some(asking));
            test();
        });
    }

    /// A screen that owns the guard, armed on `route`.
    fn armed_on(route: &str) -> u64 {
        let owner = take_ownership();
        arm_for(owner, Some(COPY), Some(route.to_owned()));
        owner
    }

    #[test]
    fn a_disarmed_guard_holds_nothing() {
        let _turn = fresh();
        with_runtime(|| {
            assert!(!hold(Held::Back, EDIT));
            assert!(!hold(Held::Push("/decks".to_string()), EDIT));
        });
    }

    #[test]
    fn an_armed_guard_holds_the_move_until_disarmed() {
        let _turn = fresh();
        with_runtime(|| {
            let owner = armed_on(EDIT);
            assert!(hold(Held::Back, EDIT), "back waits on the dialog");
            assert!(
                hold(Held::Push("/decks".to_string()), EDIT),
                "a link waits too"
            );
            assert_eq!(
                with_guard(|g| g.held.clone()).flatten(),
                Some(Held::Push("/decks".to_string())),
                "the latest move waits"
            );
            arm_for(owner, None, Some(EDIT.to_owned()));
            assert!(
                !hold(Held::Back, EDIT),
                "a saved form lets every move through"
            );
            release(owner);
        });
    }

    #[test]
    fn a_screen_that_has_gone_cannot_arm_it() {
        let _turn = fresh();
        with_runtime(|| {
            let owner = armed_on(EDIT);
            release(owner);
            assert!(!hold(Held::Back, EDIT), "the form going disarms it");
            arm_for(owner, Some(COPY), Some(EDIT.to_owned()));
            assert!(
                !hold(Held::Push("/decks".to_string()), EDIT),
                "a late reply from the gone form arms nothing"
            );
        });
    }

    #[test]
    fn a_new_screen_takes_the_guard_from_an_old_one() {
        let _turn = fresh();
        with_runtime(|| {
            let old = take_ownership();
            let new = take_ownership();
            arm_for(old, Some(COPY), Some(EDIT.to_owned()));
            assert!(!hold(Held::Back, EDIT), "only the owner arms it");
            arm_for(new, Some(COPY), Some(EDIT.to_owned()));
            release(old);
            assert!(
                hold(Held::Back, EDIT),
                "the old screen going leaves the new one's guard armed"
            );
            release(new);
        });
    }

    #[test]
    fn a_move_from_another_route_disarms_it() {
        let _turn = fresh();
        with_runtime(|| {
            let owner = armed_on(EDIT);
            assert!(
                !hold(Held::Push("/decks".to_string()), "/decks/1"),
                "armed on the form, not here"
            );
            assert!(!hold(Held::Back, EDIT), "and it stays disarmed");
            release(owner);
        });
    }

    #[test]
    fn with_no_dialog_to_ask_the_move_goes_through() {
        let _turn = fresh();
        with_runtime(|| {
            with_guard(|g| g.asking = None);
            let owner = armed_on(EDIT);
            assert!(!hold(Held::Back, EDIT));
            release(owner);
        });
    }

    #[test]
    fn a_tap_handled_on_another_thread_finds_the_guard_armed() {
        let _turn = fresh();
        with_runtime(|| {});
        let owner = armed_on(EDIT);
        let held = thread::spawn(|| {
            with_guard(|g| g.hold(Held::Push("/decks".to_owned()), EDIT).is_some())
        })
        .join()
        .unwrap();
        assert_eq!(
            held,
            Some(true),
            "Android handles taps off the render thread"
        );
        release(owner);
    }

    /// The routes the router tests run on: a deck, and its edit form under
    /// the layout that mounts the dialog, as the app's router has them.
    #[derive(Routable, Clone, PartialEq, Debug)]
    enum Route {
        #[layout(Layout)]
        #[route("/deck")]
        Deck {},
        #[route("/deck/edit")]
        Edit {},
    }

    #[component]
    fn Layout() -> Element {
        rsx! {
            Outlet::<Route> {}
            LeaveDialog {}
        }
    }

    #[component]
    fn Deck() -> Element {
        rsx! { "deck" }
    }

    thread_local! {
        /// The edit form's header back and the OS back, handed to the test.
        static BACKS: RefCell<Option<(Callback<()>, Callback<()>)>> = const { RefCell::new(None) };
    }

    /// An edit form with unsaved changes. Its header back arms the guard
    /// and pushes the deck, as the deck form's Back does; the OS back is the
    /// router's go back, as the back handler calls it.
    #[component]
    fn Edit() -> Element {
        let guard = use_leave_guard(COPY);
        let navigator = use_navigator();
        let header_back = use_callback(move |()| {
            guard.set(true);
            navigator.push(Route::Deck {});
        });
        let os_back = use_callback(move |()| navigator.go_back());
        use_hook(move || {
            guard.set(true);
            BACKS.with(|b| *b.borrow_mut() = Some((header_back, os_back)));
        });
        rsx! { "edit" }
    }

    thread_local! {
        /// The history the router tests run on, kept to read the route on view.
        static HISTORY: RefCell<Option<Rc<dyn History>>> = const { RefCell::new(None) };
    }

    fn app() -> Element {
        use_hook(GuardedHistory::provide);
        rsx! {
            Router::<Route> {}
        }
    }

    /// The app on the edit form, opened from the deck, with its two backs.
    fn on_the_edit_form() -> (VirtualDom, Callback<()>, Callback<()>) {
        let mut dom = VirtualDom::new(app);
        let history: Rc<dyn History> =
            Rc::new(dioxus::history::MemoryHistory::with_initial_path("/deck"));
        history.push("/deck/edit".to_owned());
        HISTORY.with(|h| *h.borrow_mut() = Some(Rc::clone(&history)));
        dom.provide_root_context(history);
        dom.rebuild_in_place();
        let (header_back, os_back) = BACKS.with(|b| b.borrow_mut().take()).unwrap();
        (dom, header_back, os_back)
    }

    fn route_on_view() -> String {
        HISTORY
            .with(|h| h.borrow().as_ref().map(|history| history.current_route()))
            .unwrap()
    }

    fn asking() -> bool {
        with_guard(|g| g.asking.is_some_and(|a| *a.peek())).unwrap()
    }

    #[test]
    fn the_header_back_asks_before_leaving_unsaved_changes() {
        let _turn = fresh();
        let (mut dom, header_back, _) = on_the_edit_form();
        dom.in_runtime(|| header_back.call(()));
        dom.render_immediate_to_vec();
        assert_eq!(route_on_view(), "/deck/edit", "the form stays on view");
        assert!(asking(), "the dialog asks");
        assert_eq!(
            with_guard(|g| g.held.clone()).flatten(),
            Some(Held::Push("/deck".to_owned())),
            "the header's move waits on the answer"
        );
    }

    #[test]
    fn the_os_back_asks_before_leaving_unsaved_changes() {
        let _turn = fresh();
        let (mut dom, _, os_back) = on_the_edit_form();
        dom.in_runtime(|| os_back.call(()));
        dom.render_immediate_to_vec();
        assert_eq!(route_on_view(), "/deck/edit", "the form stays on view");
        assert!(asking(), "the dialog asks");
        assert_eq!(with_guard(|g| g.held.clone()).flatten(), Some(Held::Back));
    }
}
