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
//! Leave disarms the guard and makes the move.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

use dioxus::history::History;
use dioxus::prelude::*;

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
    /// The title.
    pub title: &'static str,
    /// The body.
    pub body: &'static str,
    /// The button that stays.
    pub stay: &'static str,
    /// The button that leaves.
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
    /// The dialog's open state, from the mounted [`LeaveDialog`].
    asking: Cell<Option<Signal<bool>>>,
    /// Whether a screen is armed, which the mounted [`LeaveDialog`] marks for
    /// the unload prompt.
    armed: Cell<Option<Signal<bool>>>,
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
        self.inner.updater(Arc::new(move || {
            if !undo_pop() {
                callback();
            }
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

/// The dialog the guard asks through, over every layer a screen can have
/// open, with the kit's alert dialog's look. Mount it once inside the
/// router.
#[component]
pub fn LeaveDialog() -> Element {
    let navigator = use_navigator();
    let mut asking = use_hook(|| {
        let asking = Signal::new(false);
        GUARD.with(|g| g.asking.set(Some(asking)));
        asking
    });
    let armed = use_hook(|| {
        let armed = Signal::new(GUARD.with(|g| g.copy.get().is_some()));
        GUARD.with(|g| g.armed.set(Some(armed)));
        armed
    });
    use_drop(|| {
        GUARD.with(|g| {
            g.asking.set(None);
            g.armed.set(None);
        });
    });
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
