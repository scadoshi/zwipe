//! A full-screen picker's backdrop, mounted only while it shows.
//!
//! The deck form's pickers stay mounted and cross-fade on `.show`, and each
//! draws its own `GridCells` canvas the size of the screen. The canvas mounts
//! as the picker opens and stays through its fade out, so a closed picker
//! holds none.

use std::time::Duration;

use dioxus::prelude::*;
use tokio::time;

/// The picker's 0.2 s fade (`.swipe-select-screen`), and a frame over.
const FADE_OUT: Duration = Duration::from_millis(250);

/// Whether a picker that is `open` still shows: while open, and through its
/// fade out once closed.
pub fn use_held_open(open: ReadSignal<bool>) -> bool {
    let mut held = use_signal(|| *open.peek());
    use_effect(move || {
        if open() {
            held.set(true);
        } else if *held.peek() {
            spawn(async move {
                time::sleep(FADE_OUT).await;
                if !*open.peek() {
                    held.set(false);
                }
            });
        }
    });
    open() || held()
}
