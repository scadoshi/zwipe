//! Logging out: tell the server, then forget the session whatever it says.
//!
//! The refresh token lives in an `HttpOnly` cookie the page cannot delete:
//! only the server's answer to a logout expires it. So logout tries the
//! token route, then the cookie alone, and when neither reaches the server it
//! leaves a mark that the next start, or the browser coming back online,
//! sends again ([`PendingLogout`]).

use std::future::Future;
use std::time::Duration;

use dioxus::prelude::*;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::outbound::pending_logout::PendingLogout;
use crate::outbound::session_store::SessionStore;
use crate::outbound::storage::Storage;
use crate::outbound::timer;

/// How long logout waits on the server before clearing the session anyway.
pub const LOGOUT_TIMEOUT: Duration = Duration::from_secs(5);

/// Fires when the browser comes back online.
const ONLINE_JS: &str = "window.addEventListener('online', () => dioxus.send(true));";

/// Asks the server to end the session, then clears it here whether the
/// server answered, failed or took longer than [`LOGOUT_TIMEOUT`]. The auth
/// gate then shows Login. No toast either way; a logout the server did not
/// hear is sent again later.
pub fn logout(session: Signal<Option<Session>>, client: Signal<ZwipeClient>) {
    spawn(async move {
        let with_token = async {
            match timer::timeout(LOGOUT_TIMEOUT, invalidate_on_server(session, client)).await {
                Some(sent) => sent.map_err(|e| e.to_string()),
                None => Err(format!("timed out after {LOGOUT_TIMEOUT:?}")),
            }
        };
        let by_cookie = || by_cookie(client);
        if !tell_server(&PendingLogout::browser(), with_token, by_cookie).await {
            tracing::warn!("server-side logout did not land; it goes again on the next start");
        }
        clear_local(session);
    });
}

/// Sends a logout the server has not heard yet, once as the app starts and
/// again each time the browser comes back online, until one lands.
pub fn use_pending_logout(client: Signal<ZwipeClient>) {
    use_future(move || async move {
        let pending = PendingLogout::browser();
        let _ = resend_pending(&pending, || by_cookie(client)).await;
        let mut online = document::eval(ONLINE_JS);
        while online.recv::<bool>().await.is_ok() {
            if pending.is_marked() {
                let _ = resend_pending(&pending, || by_cookie(client)).await;
            }
        }
    });
}

/// Tells the server the session is over: by `with_token`, else by the cookie
/// alone. Neither reaching it marks the logout pending. Answers whether the
/// server heard.
pub(crate) async fn tell_server<S, B, F>(
    pending: &PendingLogout<S>,
    with_token: impl Future<Output = Result<(), String>>,
    by_cookie: B,
) -> bool
where
    S: Storage,
    B: FnOnce() -> F,
    F: Future<Output = Result<(), String>>,
{
    if let Err(e) = with_token.await {
        tracing::warn!("server-side logout failed: {e}; trying the cookie alone");
        if let Err(e) = by_cookie().await {
            tracing::warn!("logout by the cookie failed: {e}");
            pending.mark();
            return false;
        }
    }
    true
}

/// Sends a pending logout by the cookie alone when one is marked, clearing
/// the mark once it lands. Answers whether nothing is left pending.
pub(crate) async fn resend_pending<S, B, F>(pending: &PendingLogout<S>, by_cookie: B) -> bool
where
    S: Storage,
    B: FnOnce() -> F,
    F: Future<Output = Result<(), String>>,
{
    if !pending.is_marked() {
        return true;
    }
    match by_cookie().await {
        Ok(()) => {
            pending.clear();
            true
        }
        Err(e) => {
            tracing::warn!("pending logout still did not land: {e}");
            false
        }
    }
}

/// Ends the browser's session by its cookie alone, giving up after
/// [`LOGOUT_TIMEOUT`].
async fn by_cookie(client: Signal<ZwipeClient>) -> Result<(), String> {
    let http = client.peek().clone();
    match timer::timeout(LOGOUT_TIMEOUT, http.logout_cookie_alone()).await {
        Some(sent) => sent.map_err(|e| e.to_string()),
        None => Err(format!("timed out after {LOGOUT_TIMEOUT:?}")),
    }
}

/// Refreshes the session if it needs it, then sends the logout request.
async fn invalidate_on_server(
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
) -> Result<(), ClientError> {
    let current = ensure_fresh(session, client).await?;
    let http = client.peek().clone();
    http.logout_cookie(&current).await
}

/// Drops the stored session and the signal.
fn clear_local(mut session: Signal<Option<Session>>) {
    SessionStore::browser().infallible_delete();
    session.set(None);
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::outbound::pending_logout::PendingLogout;
    use crate::outbound::storage::MemoryStorage;

    fn offline() -> String {
        "offline".to_string()
    }

    #[tokio::test]
    async fn a_logout_the_token_route_misses_goes_by_the_cookie() {
        let pending = PendingLogout::new(MemoryStorage::default());
        let by_cookie = Cell::new(0);
        let told = tell_server(&pending, async { Err(offline()) }, || async {
            by_cookie.set(by_cookie.get() + 1);
            Ok(())
        })
        .await;
        assert!(told, "the cookie route ended it");
        assert_eq!(by_cookie.get(), 1);
        assert!(!pending.is_marked(), "nothing left to tell");
    }

    #[tokio::test]
    async fn a_logout_no_route_reaches_waits_for_the_next_start() {
        let pending = PendingLogout::new(MemoryStorage::default());
        let told = tell_server(&pending, async { Err(offline()) }, || async {
            Err(offline())
        })
        .await;
        assert!(!told);
        assert!(pending.is_marked(), "the next start tells the server");

        assert!(!resend_pending(&pending, || async { Err(offline()) }).await);
        assert!(pending.is_marked(), "still offline: kept");
        assert!(resend_pending(&pending, || async { Ok(()) }).await);
        assert!(!pending.is_marked(), "heard: cleared");
    }

    #[tokio::test]
    async fn a_logout_the_token_route_ends_sends_nothing_more() {
        let pending = PendingLogout::new(MemoryStorage::default());
        let told = tell_server(&pending, async { Ok(()) }, || async {
            panic!("the cookie route is not needed")
        })
        .await;
        assert!(told);
        assert!(!pending.is_marked());
    }
}
