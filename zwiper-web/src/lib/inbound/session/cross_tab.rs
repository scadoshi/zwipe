//! The browser-wide refresh lock: tabs take turns refreshing, so they rarely
//! present the shared refresh cookie at once. A Web Lock (`navigator.locks`)
//! held from JavaScript until Rust lets it go, or for at most
//! `HOLD_LIMIT_MS`, so a lost handle never wedges the other tabs. A tab
//! waits at most `WAIT_LIMIT_MS` for it and then refreshes anyway, as it
//! does where the browser has no Web Locks: the server answers a refresh
//! token presented twice within its reuse window (`REFRESH_REUSE_SECONDS`,
//! 30 seconds by default) with the same successor, which covers the overlap.

use dioxus::document::Eval;
use dioxus::prelude::*;

/// The lock every tab of the app asks for.
const LOCK_NAME: &str = "zwipe-refresh";

/// Longest a tab waits for the lock before refreshing without it, in
/// milliseconds, so another tab's slow refresh never holds this tab's first
/// paint.
const WAIT_LIMIT_MS: u32 = 2_000;

/// Longest a tab holds the lock, in milliseconds, whatever happens to it.
/// The server's reuse window covers a refresh still out when it lapses.
const HOLD_LIMIT_MS: u32 = 5_000;

// First paint waits at most 2s on another tab, and a tab that waited out the
// lock and the hold still presents the cookie inside the server's reuse
// window (REFRESH_REUSE_SECONDS, 30 by default).
const _: () = assert!(WAIT_LIMIT_MS <= 2_000);
const _: () = assert!(WAIT_LIMIT_MS + HOLD_LIMIT_MS < 30_000);

/// Requests the lock for at most `WAIT_LIMIT_MS`, says whether it is held
/// (`false` with no Web Locks or once the wait runs out), then holds it until
/// Rust sends anything or `HOLD_LIMIT_MS` passes.
fn lock_js() -> String {
    format!(
        r"
if (!navigator.locks) {{
    dioxus.send(false);
}} else {{
    const wait = new AbortController();
    const timer = setTimeout(() => wait.abort(), {WAIT_LIMIT_MS});
    let granted = false;
    try {{
        await navigator.locks.request('{LOCK_NAME}', {{ signal: wait.signal }}, async () => {{
            granted = true;
            clearTimeout(timer);
            dioxus.send(true);
            await Promise.race([
                dioxus.recv(),
                new Promise((resolve) => setTimeout(resolve, {HOLD_LIMIT_MS})),
            ]);
        }});
    }} catch (e) {{
        if (!granted) dioxus.send(false);
    }}
}}
"
    )
}

/// The held lock, let go when it drops, on every path out of the refresh.
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
    /// Waits up to `WAIT_LIMIT_MS` for the browser-wide lock. Holds nothing
    /// where the browser has no Web Locks, the wait runs out or the bridge
    /// fails, and the caller refreshes anyway.
    pub async fn acquire() -> Self {
        let mut eval = document::eval(&lock_js());
        let held = matches!(eval.recv::<bool>().await, Ok(true));
        if !held {
            tracing::debug!("refreshing without the cross-tab lock");
        }
        Self {
            held: held.then_some(eval),
        }
    }
}

impl Drop for CrossTabLock {
    /// Lets the next tab in.
    fn drop(&mut self) {
        if let Some(eval) = self.held.take() {
            // A failed send leaves the lock to the hold limit.
            let _ = eval.send(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tab_waits_briefly_then_refreshes_without_the_lock() {
        let js = lock_js();
        assert!(
            js.contains(&format!("setTimeout(() => wait.abort(), {WAIT_LIMIT_MS})")),
            "{js}"
        );
        assert!(js.contains("signal: wait.signal"), "{js}");
        assert!(js.contains("if (!granted) dioxus.send(false)"), "{js}");
    }

    #[test]
    fn a_held_lock_lets_go_after_the_hold_limit() {
        let js = lock_js();
        assert!(
            js.contains(&format!("setTimeout(resolve, {HOLD_LIMIT_MS})")),
            "{js}"
        );
    }
}
