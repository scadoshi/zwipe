//! Asks before a screen with unsaved changes is left. A screen arms the
//! guard while it holds changes ([`use_leave_guard`]); while it is armed,
//! every way out waits on [`LeaveDialog`]:
//!
//! - the router's moves (a link, the nav, a back arrow, a pushed route),
//!   which [`GuardedHistory`] holds back;
//! - the browser's back and forward, which it undoes by putting the
//!   screen's URL back as a new entry;
//! - a reload or a closed tab, which the browser asks about itself.
//!
//! Stay drops the move and leaves every layer on the screen as it was.
//! Leave disarms the guard and makes the move. Every move it lets through
//! then goes past `fresh_build::after_move`, which loads the page in full
//! once a newer build is live.
//!
//! The browser's back or forward with a sheet or dialog open closes the top
//! one instead, armed or not, and puts the screen's URL back; the guard only
//! asks once nothing is open.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

use components::OverlayBackStack;
use dioxus::history::History;
use dioxus::prelude::*;

use crate::inbound::chrome::fresh_build;
use crate::inbound::router::Route;

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
    /// The browser's back or forward, already undone: the screen's URL went
    /// back on as a new entry, so going back from it lands where the
    /// browser went.
    Popped,
    /// A page outside the app.
    External(String),
}

/// What the dialog says, from the screen that armed the guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveCopy {
    /// The dialog's heading, the question it asks.
    pub title: &'static str,
    /// The line under it: what leaving loses.
    pub body: &'static str,
    /// The label of the button that keeps the screen and its changes.
    pub stay: &'static str,
    /// The label of the button that drops the changes and makes the move.
    pub leave: &'static str,
}

/// The guard, one per tab. The history the router calls and the popstate
/// listener it registers run outside any component, so the state lives here.
#[derive(Default)]
struct Guard {
    /// The armed screen's copy; `None` while disarmed.
    copy: Cell<Option<LeaveCopy>>,
    /// The armed screen's route, which an undone back or forward puts back.
    route: RefCell<String>,
    /// The move waiting on the dialog.
    held: RefCell<Option<Held>>,
    /// The platform's history, under [`GuardedHistory`].
    inner: RefCell<Option<Rc<dyn History>>>,
    /// The dialog's open state, from the first [`LeaveDialog`] to mount.
    asking: Cell<Option<Signal<bool>>>,
    /// Whether a screen is armed, which every mounted [`LeaveDialog`] marks
    /// for the unload prompt.
    armed: Cell<Option<Signal<bool>>>,
    /// The open sheets and dialogs, from [`watch_overlays`].
    overlays: Cell<Option<OverlayBackStack>>,
    /// The route on screen, which a back or forward that closes an overlay
    /// puts back.
    shown: RefCell<String>,
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

/// On the browser's back or forward while armed, puts the screen's URL back
/// and holds the move. Returns whether it did.
fn undo_pop() -> bool {
    let inner = GUARD.with(|g| {
        g.copy.get()?;
        g.inner
            .borrow()
            .clone()
            .map(|inner| (inner, g.route.borrow().clone()))
    });
    let Some((inner, route)) = inner else {
        return false;
    };
    inner.push(route);
    hold(Held::Popped)
}

/// On the browser's back or forward with a sheet or dialog open, puts the
/// screen's URL back and closes the top one. Returns whether it did.
fn close_overlay_on_pop() -> bool {
    let found = GUARD.with(|g| {
        let overlays = g.overlays.get()?;
        let inner = g.inner.borrow().clone()?;
        Some((overlays, inner, g.shown.borrow().clone()))
    });
    let Some((mut overlays, inner, shown)) = found else {
        return false;
    };
    if !overlays.close_top() {
        return false;
    }
    inner.push(shown);
    true
}

/// Whether a screen holds unsaved changes.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(dead_code, reason = "the browser build's fresh-build check reads it")
)]
pub(crate) fn is_armed() -> bool {
    GUARD.with(|g| g.copy.get().is_some())
}

/// Records the platform's current route as the one on screen.
fn show_current() {
    GUARD.with(|g| {
        if let Some(inner) = g.inner.borrow().as_ref() {
            *g.shown.borrow_mut() = inner.current_route();
        }
    });
}

/// Records `route` as the one on screen.
fn show(route: String) {
    GUARD.with(|g| *g.shown.borrow_mut() = route);
}

/// Lets the browser's back and forward close the top of `overlays` before
/// they move or ask. Call it once, where the stack is provided.
pub fn watch_overlays(overlays: OverlayBackStack) {
    GUARD.with(|g| g.overlays.set(Some(overlays)));
}

/// Arms the guard with `copy`, recording the route it guards, or disarms
/// it with `None`.
fn arm(copy: Option<LeaveCopy>) {
    GUARD.with(|g| {
        if copy.is_some()
            && g.copy.get().is_none()
            && let Some(inner) = g.inner.borrow().as_ref()
        {
            *g.route.borrow_mut() = inner.current_route();
        }
        g.copy.set(copy);
        if let Some(mut armed) = g.armed.get()
            && *armed.peek() != copy.is_some()
        {
            armed.set(copy.is_some());
        }
    });
}

/// The platform's history with the guard in front: while a screen is armed,
/// each move the router makes is held for the dialog instead.
pub struct GuardedHistory {
    inner: Rc<dyn History>,
}

impl std::fmt::Debug for GuardedHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GuardedHistory").finish_non_exhaustive()
    }
}

impl GuardedHistory {
    /// Puts the guard in front of `inner`.
    pub fn new(inner: Rc<dyn History>) -> Self {
        show(inner.current_route());
        GUARD.with(|g| *g.inner.borrow_mut() = Some(Rc::clone(&inner)));
        Self { inner }
    }

    /// Provides the guard in front of the platform's history to the calling
    /// component and everything under it. Call it once above the router.
    pub fn provide() {
        let guarded: Rc<dyn History> = Rc::new(Self::new(dioxus::history::history()));
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
            self.inner.push(route.clone());
            show(route);
            fresh_build::after_move();
        }
    }

    fn replace(&self, path: String) {
        if !hold(Held::Replace(path.clone())) {
            self.inner.replace(path.clone());
            show(path);
            fresh_build::after_move();
        }
    }

    fn external(&self, url: String) -> bool {
        hold(Held::External(url.clone())) || self.inner.external(url)
    }

    fn updater(&self, callback: Arc<dyn Fn() + Send + Sync>) {
        self.inner.updater(Arc::new(move || {
            if close_overlay_on_pop() || undo_pop() {
                return;
            }
            show_current();
            callback();
            fresh_build::after_move();
        }));
    }

    fn include_prevent_default(&self) -> bool {
        self.inner.include_prevent_default()
    }
}

/// The guard as a screen holds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveGuard;

impl LeaveGuard {
    /// Disarms the guard for a move the screen makes itself, such as after
    /// a save.
    pub fn release(self) {
        arm(None);
    }
}

/// Arms the guard with `copy` while `changed` is true, and disarms it when
/// the screen goes.
pub fn use_leave_guard(changed: Memo<bool>, copy: LeaveCopy) -> LeaveGuard {
    use_effect(move || arm(changed().then_some(copy)));
    use_drop(|| arm(None));
    LeaveGuard
}

/// The browser's own prompt on a reload or a closed tab, while the marker
/// [`LeaveDialog`] draws says a screen is armed.
const UNLOAD_JS: &str = r#"
if (!window.__zwipeLeaveUnload) {
    window.__zwipeLeaveUnload = true;
    window.addEventListener('beforeunload', (e) => {
        if (document.querySelector('.web-leave-guard[data-armed="true"]')) {
            e.preventDefault();
            e.returnValue = '';
        }
    });
}
"#;

/// Keeps Tab on the dialog's two buttons while it is open.
const TAB_JS: &str = r"
if (!window.__zwipeLeaveTab) {
    window.__zwipeLeaveTab = true;
    document.addEventListener('keydown', (e) => {
        if (e.key !== 'Tab') return;
        const dialog = document.querySelector('.web-leave-dialog');
        if (!dialog) return;
        const buttons = Array.from(dialog.querySelectorAll('button'));
        const at = buttons.indexOf(document.activeElement);
        e.preventDefault();
        const next = (at + (e.shiftKey ? -1 : 1) + buttons.length) % buttons.length;
        buttons[next].focus();
    }, true);
}
";

/// Remembers what had focus and puts it on Stay.
const FOCUS_STAY_JS: &str = "(() => { const stay = document.querySelector('.web-leave-dialog .web-leave-stay'); if (!stay) return; window.__zwipeLeaveFrom = document.activeElement; stay.focus(); })()";

/// Gives focus back to what had it before the dialog.
const FOCUS_BACK_JS: &str = "(() => { const from = window.__zwipeLeaveFrom; window.__zwipeLeaveFrom = null; if (from && from.isConnected && from.focus) from.focus(); })()";

/// The guard's dialog state, `(asking, armed)`, which every mounted
/// [`LeaveDialog`] shares. The first one to mount creates it, owned by the
/// root scope, so it outlives any one dialog: a frame that replaces another
/// mounts its dialog before the last one drops, and the guard writes the
/// state from whichever screen or nav button holds a move.
fn use_dialog_slots() -> (Signal<bool>, Signal<bool>) {
    use_hook(|| {
        GUARD.with(|g| {
            let asking = g.asking.get().unwrap_or_else(|| {
                let asking = Signal::new_in_scope(false, ScopeId::ROOT);
                g.asking.set(Some(asking));
                asking
            });
            let armed = g.armed.get().unwrap_or_else(|| {
                let armed = Signal::new_in_scope(g.copy.get().is_some(), ScopeId::ROOT);
                g.armed.set(Some(armed));
                armed
            });
            (asking, armed)
        })
    })
}

#[cfg(test)]
thread_local! {
    /// The scopes of the mounted [`LeaveDialog`]s, oldest first.
    static MOUNTED_DIALOGS: RefCell<Vec<ScopeId>> = const { RefCell::new(Vec::new()) };
}

/// The scope of the newest mounted [`LeaveDialog`], which names it: a frame
/// drawn again mounts a dialog in a new scope.
#[cfg(test)]
pub(crate) fn mounted_dialog() -> Option<ScopeId> {
    MOUNTED_DIALOGS.with(|m| m.borrow().last().copied())
}

/// Records the calling [`LeaveDialog`] in [`MOUNTED_DIALOGS`] while it is
/// mounted.
#[cfg(test)]
fn use_mounted_record() {
    let scope = use_hook(|| {
        let scope = dioxus::core::current_scope_id();
        MOUNTED_DIALOGS.with(|m| m.borrow_mut().push(scope));
        scope
    });
    use_drop(move || MOUNTED_DIALOGS.with(|m| m.borrow_mut().retain(|s| *s != scope)));
}

/// The dialog the guard asks through, over every layer a screen can have
/// open, with the kit's alert dialog's look. Mount it once inside the
/// router.
#[component]
pub fn LeaveDialog() -> Element {
    let navigator = use_navigator();
    let (mut asking, armed) = use_dialog_slots();
    #[cfg(test)]
    use_mounted_record();
    use_hook(|| {
        document::eval(UNLOAD_JS);
        document::eval(TAB_JS);
    });
    use_effect(move || {
        if asking() {
            document::eval(FOCUS_STAY_JS);
        }
    });

    let mut stay = move || {
        GUARD.with(|g| g.held.borrow_mut().take());
        asking.set(false);
        document::eval(FOCUS_BACK_JS);
    };
    let mut leave = move || {
        let held = GUARD.with(|g| g.held.borrow_mut().take());
        arm(None);
        asking.set(false);
        let to_route = |path: &str| Route::from_str(path).unwrap_or(Route::Home {});
        match held {
            Some(Held::Push(path)) => {
                navigator.push(to_route(&path));
            }
            Some(Held::Replace(path)) => {
                navigator.replace(to_route(&path));
            }
            Some(Held::Back | Held::Popped) => navigator.go_back(),
            Some(Held::Forward) => navigator.go_forward(),
            Some(Held::External(url)) => {
                navigator.push(NavigationTarget::<Route>::External(url));
            }
            None => {}
        }
    };

    let copy = GUARD.with(|g| g.copy.get());
    rsx! {
        div { class: "web-leave-guard", hidden: true, "data-armed": "{armed}" }
        if let (true, Some(copy)) = (asking(), copy) {
            div {
                class: "alert-dialog-overlay web-leave-overlay",
                "data-state": "open",
                onclick: move |_| stay(),
            }
            div {
                class: "alert-dialog web-leave-dialog",
                role: "alertdialog",
                aria_modal: "true",
                "aria-labelledby": "web-leave-title",
                "aria-describedby": "web-leave-body",
                "data-state": "open",
                h2 { id: "web-leave-title", class: "alert-dialog-title", "{copy.title}" }
                hr { class: "alert-dialog-rule" }
                div { id: "web-leave-body", class: "alert-dialog-description", "{copy.body}" }
                hr { class: "alert-dialog-rule" }
                div { class: "alert-dialog-actions",
                    button {
                        class: "alert-dialog-cancel web-leave-stay",
                        r#type: "button",
                        onclick: move |_| stay(),
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
    use dioxus::history::MemoryHistory;

    use super::*;

    const COPY: LeaveCopy = LeaveCopy {
        title: "Leave without saving?",
        body: "Your changes are lost.",
        stay: "Stay",
        leave: "Leave",
    };

    thread_local! {
        /// The open sheet's signal, handed out to the test body.
        static OPEN: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
    }

    #[component]
    fn Root() -> Element {
        let overlays = components::use_overlay_back_stack();
        use_context_provider(|| overlays);
        watch_overlays(overlays);
        rsx! { Sheet {} }
    }

    #[component]
    fn Sheet() -> Element {
        let open = use_signal(|| true);
        components::use_overlay_back(open);
        OPEN.with(|o| *o.borrow_mut() = Some(open));
        rsx! {}
    }

    thread_local! {
        /// Which of the two dialogs are mounted: the first, the second.
        static MOUNTED: RefCell<Option<Signal<(bool, bool)>>> = const { RefCell::new(None) };
        /// The second dialog's open state, handed out to the test body.
        static SECOND: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
    }

    #[component]
    fn TwoFrames() -> Element {
        let mounted = use_signal(|| (true, false));
        MOUNTED.with(|m| *m.borrow_mut() = Some(mounted));
        let (first, second) = mounted();
        rsx! {
            if first {
                Slots { second: false }
            }
            if second {
                Slots { second: true }
            }
        }
    }

    #[component]
    fn Slots(second: bool) -> Element {
        let (asking, _) = use_dialog_slots();
        if second {
            SECOND.with(|s| *s.borrow_mut() = Some(asking));
        }
        rsx! {}
    }

    #[test]
    fn a_dialog_dropped_after_the_next_one_mounts_leaves_the_guard_its_successor() {
        let mut dom = VirtualDom::new(TwoFrames);
        dom.rebuild_in_place();
        let mut mounted = MOUNTED.with(|m| *m.borrow()).unwrap();

        // About's frame to the authed one: the new nav's dialog mounts, then
        // the old one goes.
        dom.in_runtime(|| mounted.set((true, true)));
        dom.render_immediate_to_vec();
        dom.in_runtime(|| mounted.set((false, true)));
        dom.render_immediate_to_vec();

        let second = SECOND.with(|s| *s.borrow()).unwrap();
        assert_eq!(
            GUARD.with(|g| g.asking.get()),
            Some(second),
            "the guard opens the dialog still on screen"
        );
        assert!(
            GUARD.with(|g| g.armed.get()).is_some(),
            "the unload marker survives"
        );

        arm(Some(COPY));
        assert!(hold(Held::Back), "an armed guard holds the move");
        assert!(dom.in_runtime(|| *second.peek()), "and the dialog opens");
        GUARD.with(|g| g.held.borrow_mut().take());
        arm(None);
    }

    #[test]
    fn the_dialog_signals_belong_to_the_root_and_outlive_any_dialog() {
        let mut dom = VirtualDom::new(TwoFrames);
        dom.rebuild_in_place();
        let mut mounted = MOUNTED.with(|m| *m.borrow()).unwrap();
        dom.in_runtime(|| mounted.set((false, false)));
        dom.render_immediate_to_vec();

        let asking = GUARD
            .with(|g| g.asking.get())
            .expect("the slot stays filled");
        let armed = GUARD
            .with(|g| g.armed.get())
            .expect("the slot stays filled");
        assert_eq!(asking.origin_scope(), ScopeId::ROOT, "asking is the root's");
        assert_eq!(armed.origin_scope(), ScopeId::ROOT, "armed is the root's");
        assert!(
            dom.in_runtime(|| asking.try_peek().is_ok() && armed.try_peek().is_ok()),
            "both still read with no dialog mounted"
        );
    }

    #[test]
    fn the_browser_back_closes_an_open_sheet_before_the_guard_asks() {
        let inner: Rc<dyn History> = Rc::new(MemoryHistory::with_initial_path("/decks"));
        let guarded = GuardedHistory::new(Rc::clone(&inner));
        guarded.push("/decks/new".to_owned());
        arm(Some(COPY));
        let mut dom = VirtualDom::new(Root);
        dom.rebuild_in_place();
        dom.render_immediate_to_vec();
        let open = OPEN.with(|o| *o.borrow()).unwrap();

        inner.go_back();
        let closed = dom.in_runtime(close_overlay_on_pop);

        assert!(closed, "the pop closed the sheet");
        assert!(!dom.in_runtime(|| *open.peek()), "the sheet is shut");
        assert_eq!(
            inner.current_route(),
            "/decks/new",
            "the form's URL is back"
        );
        assert!(
            GUARD.with(|g| g.held.borrow().is_none()),
            "the guard did not ask"
        );

        inner.go_back();
        assert!(
            !dom.in_runtime(close_overlay_on_pop),
            "nothing open, so the guard's turn"
        );
        assert!(undo_pop(), "the guard holds the next back");
        arm(None);
    }
}
