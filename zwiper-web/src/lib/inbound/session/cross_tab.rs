//! The browser-wide refresh lock: one tab refreshes at a time, so tabs never
//! race the shared refresh cookie. A Web Lock (`navigator.locks`) held from
//! JavaScript until Rust lets it go, or for at most 20 seconds, so a
//! lost handle never wedges the other tabs. Without Web Locks every tab goes
//! ahead, and the server's reuse window covers the race.

use dioxus::document::Eval;
use dioxus::prelude::*;

/// The lock every tab of the app asks for.
const LOCK_NAME: &str = "zwipe-refresh";

/// Longest a tab holds the lock, in milliseconds, whatever happens to it.
const HOLD_LIMIT_MS: u32 = 20_000;

/// Requests the lock, says when it is held (or that there is none), then
/// holds it until Rust sends anything or the limit passes.
const LOCK_JS: &str = r"
if (!navigator.locks) {
    dioxus.send(false);
} else {
    await navigator.locks.request('__NAME__', async () => {
        dioxus.send(true);
        await Promise.race([
            dioxus.recv(),
            new Promise((resolve) => setTimeout(resolve, __LIMIT__)),
        ]);
    });
}
";

/// The held lock; dropping it without [`release`](Self::release) leaves it
/// to the time limit.
pub struct CrossTabLock {
    held: Option<Eval>,
}

impl std::fmt::Debug for CrossTabLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CrossTabLock")
            .field("held", &self.held.is_some())
            .finish()
    }
}

impl CrossTabLock {
    /// Waits for the browser-wide lock. Holds nothing where the browser has
    /// no Web Locks or the bridge fails.
    pub async fn acquire() -> Self {
        let script = LOCK_JS
            .replace("__NAME__", LOCK_NAME)
            .replace("__LIMIT__", &HOLD_LIMIT_MS.to_string());
        let mut eval = document::eval(&script);
        let held = matches!(eval.recv::<bool>().await, Ok(true));
        Self {
            held: held.then_some(eval),
        }
    }

    /// Lets the next tab in.
    pub fn release(mut self) {
        if let Some(eval) = self.held.take() {
            let _ = eval.send(true);
        }
    }
}
