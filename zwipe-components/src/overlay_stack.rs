//! Back-aware overlay stack, for the apps.
//!
//! The OS back intent (the iOS edge swipe, Android's hardware or gesture
//! back) is bridged by each app's back handler, which normally routes it to
//! the router's `go_back`. Overlays, anything shown on top of the current
//! screen without a route change (bottom sheets, dialogs, in-place overlay
//! screens), are not routes, so a raw `go_back` would blow past them and
//! leave the screen. This stack lets the back intent close the top-most open
//! overlay first, falling through to the router only when none are open.
//!
//! Each overlay registers a close action while it is open, through
//! [`use_overlay_back`] for one toggled by a signal or
//! [`use_overlay_back_action`] for one that closes through a callback. The
//! back handler calls [`OverlayBackStack::close_top`]. Because closing flips
//! the overlay's open state, the registration effect then deregisters it: the
//! stack stays truthful with no manual bookkeeping.

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

/// Monotonic source of per-overlay-instance ids. A component takes its id
/// once through `use_hook`, so it stays stable across that instance's
/// re-renders and unique across instances.
static NEXT_OVERLAY_ID: AtomicU64 = AtomicU64::new(0);

fn next_overlay_id() -> u64 {
    NEXT_OVERLAY_ID.fetch_add(1, Ordering::Relaxed)
}

/// App-level LIFO of the currently open overlays, provided once above the
/// router. A `Copy` handle over a signal; each entry pairs an overlay
/// instance's id with the `Callback` that closes it.
#[derive(Clone, Copy)]
pub struct OverlayBackStack {
    entries: Signal<Vec<(u64, Callback<()>)>>,
}

/// Creates the stack. Provide it with `use_context_provider` above the router
/// so both the overlays and the back handler can reach it.
pub fn use_overlay_back_stack() -> OverlayBackStack {
    OverlayBackStack {
        entries: use_signal(Vec::new),
    }
}

impl OverlayBackStack {
    /// Registers an open overlay's close action. Re-pushing the same instance
    /// moves it to the top rather than duplicating it.
    fn push(&mut self, id: u64, close: Callback<()>) {
        let mut entries = self.entries.write();
        entries.retain(|(i, _)| *i != id);
        entries.push((id, close));
    }

    /// Removes an overlay's entry, on close or unmount. No-op if absent.
    fn remove(&mut self, id: u64) {
        self.entries.write().retain(|(i, _)| *i != id);
    }

    /// Closes the most recently opened overlay if there is one, popping it
    /// and invoking its close action. Returns `true` when an overlay was
    /// closed, so the back handler skips the router.
    pub fn close_top(&mut self) -> bool {
        let popped = self.entries.write().pop();
        if let Some((_, close)) = popped {
            close.call(());
            true
        } else {
            false
        }
    }
}

/// Keeps `close` on the stack while `is_open` is true. For an overlay that
/// closes through a callback rather than a settable signal, such as a dialog
/// whose open state its consumer drives.
pub fn use_overlay_back_action(is_open: ReadSignal<bool>, close: Callback<()>) {
    let stack: OverlayBackStack = use_context();
    let id = use_hook(next_overlay_id);

    use_effect(move || {
        let mut stack = stack;
        if is_open() {
            stack.push(id, close);
        } else {
            stack.remove(id);
        }
    });

    use_drop(move || {
        let mut stack = stack;
        stack.remove(id);
    });
}

/// Registers a signal-toggled overlay so the back intent closes it by setting
/// `open` false instead of navigating the router.
pub fn use_overlay_back(open: Signal<bool>) {
    let close = use_callback(move |()| {
        let mut open = open;
        open.set(false);
    });
    use_overlay_back_action(open.into(), close);
}
