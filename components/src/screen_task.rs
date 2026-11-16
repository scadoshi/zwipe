//! Server writes a screen starts that must land whatever the screen does next.
//!
//! A task from `spawn` belongs to the screen that started it and ends when the
//! screen goes, dropping a request it has not sent yet. [`spawn_write`] runs a
//! write in the root instead, so it is sent exactly once and runs to its end. A
//! screen's own signals end with it, so such a task checks [`Mounted`] before it
//! touches them; root state and toasts it may touch at any time.

use std::future::Future;

use dioxus::core::spawn_forever;
use dioxus::prelude::*;

/// Runs `write` in the root, so it is sent and runs to its end even when the screen
/// that started it goes first. For writes only: a read whose screen is gone has no one
/// to answer.
pub fn spawn_write(write: impl Future<Output = ()> + 'static) {
    let _task = spawn_forever(write);
}

/// Whether the screen that made it is still up, for a [`spawn_write`] task that
/// touches the screen's own signals once its answer lands. It holds a signal of the
/// screen's own, which ends with the screen, so it reads false from then on.
#[derive(Clone, Copy, Debug)]
pub struct Mounted(Signal<()>);

impl Mounted {
    /// True until the screen goes.
    #[must_use]
    pub fn get(self) -> bool {
        self.0.try_peek().is_ok()
    }
}

/// This screen's [`Mounted`].
pub fn use_mounted() -> Mounted {
    Mounted(use_signal(|| ()))
}
