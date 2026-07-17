//! Back-aware overlay stack.
//!
//! The OS back intent (iOS edge-swipe / Android hardware or gesture back) is
//! bridged by [`BackHandlerLayout`](super::back_handler), which normally routes
//! it to the router's `go_back`. But *overlays* — anything shown on top of the
//! current screen without a route change: in-place `.screen` overlays toggled by
//! an `open` signal (`SwipeSelect`, `OracleTagSelect`, the card-filter sheet) and
//! the `AlertDialog`-based dialogs — are not routes, so a raw `go_back` would blow
//! past them and exit the whole screen. This stack lets the back intent close the
//! top-most open overlay first, falling through to the router only when none are
//! open.
//!
//! Each overlay registers a **close action** while it is open (via
//! [`use_overlay_back`] for signal-toggled overlays, or [`use_overlay_back_action`]
//! for ones that close through a callback, e.g. the shared `AlertDialogRoot`
//! wrapper driving `on_open_change`). The back handler calls
//! [`OverlayBackStack::close_top`]. Because closing flips the overlay's open state,
//! the registration effect then deregisters it — the stack stays truthful with no
//! manual bookkeeping.

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

/// Monotonic source of per-overlay-instance ids. A component gets its id once via
/// `use_hook`, so it stays stable across that instance's re-renders while being
/// unique across instances (used as the stack key).
static NEXT_OVERLAY_ID: AtomicU64 = AtomicU64::new(0);

fn next_overlay_id() -> u64 {
    NEXT_OVERLAY_ID.fetch_add(1, Ordering::Relaxed)
}

/// App-level LIFO of the currently-open overlays, provided once above the router
/// in `spawn_upkeeper`. A `Copy` handle over a signal. Each entry pairs an overlay
/// instance's stable id with a `Callback` that closes it.
#[derive(Clone, Copy)]
pub struct OverlayBackStack {
    entries: Signal<Vec<(u64, Callback<()>)>>,
}

/// Creates the stack. Provide it via `use_context_provider` in `spawn_upkeeper`
/// so both the overlays and the back handler can reach it.
pub fn use_overlay_back_stack() -> OverlayBackStack {
    OverlayBackStack {
        entries: use_signal(Vec::new),
    }
}

impl OverlayBackStack {
    /// Registers an open overlay's close action. Idempotent by id: re-pushing the
    /// same instance moves it to the top rather than duplicating it.
    fn push(&mut self, id: u64, close: Callback<()>) {
        let mut entries = self.entries.write();
        entries.retain(|(i, _)| *i != id);
        entries.push((id, close));
    }

    /// Removes an overlay's entry (on close or unmount). No-op if absent.
    fn remove(&mut self, id: u64) {
        self.entries.write().retain(|(i, _)| *i != id);
    }

    /// Closes the top-most (most-recently-opened) overlay if there is one, popping
    /// it and invoking its close action. Returns `true` when an overlay was closed
    /// (so the back handler skips the router). Called from the back handler.
    #[cfg_attr(
        not(all(any(target_os = "ios", target_os = "android"), feature = "mobile")),
        allow(dead_code)
    )]
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

/// Core registration: keep `close` on the stack while `is_open` is true. Every
/// public hook funnels through this.
fn register(is_open: ReadSignal<bool>, close: Callback<()>) {
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

/// Registers a signal-toggled overlay with the [`OverlayBackStack`] while `open`
/// is true, so the OS back gesture closes it (by setting `open` false) instead of
/// navigating the router. Signal-driven overlays (`SwipeSelect`, `OracleTagSelect`,
/// the card-filter sheet) call this once near the top of their body.
pub fn use_overlay_back(open: Signal<bool>) {
    let close = use_callback(move |_: ()| {
        let mut open = open;
        open.set(false);
    });
    register(open.into(), close);
}

/// Registers an overlay that closes through a callback rather than a settable
/// signal — e.g. the shared `AlertDialogRoot` wrapper, whose open state is driven
/// by its consumer via `on_open_change`. `is_open` reflects the overlay's open
/// state; `on_close` performs the close.
pub fn use_overlay_back_action(is_open: ReadSignal<bool>, on_close: Callback<()>) {
    register(is_open, on_close);
}
