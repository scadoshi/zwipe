//! A session fit for an authed call, refreshed at most once at a time, in
//! this tab and across the browser's tabs.

use std::sync::{Arc, OnceLock};

use chrono::Utc;
use dioxus::prelude::*;
use tokio::sync::{Mutex, oneshot};
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::session::{self, Freshness};

use crate::inbound::session::cross_tab::CrossTabLock;
use crate::outbound::session_store::SessionStore;

/// The single-flight lock: at most one refresh request out. A caller that
/// loses the race waits, then finds the winner's session.
static REFRESH_LOCK: OnceLock<Arc<Mutex<()>>> = OnceLock::new();

fn refresh_lock() -> Arc<Mutex<()>> {
    Arc::clone(REFRESH_LOCK.get_or_init(|| Arc::new(Mutex::new(()))))
}

/// Returns a session whose access token is good for at least
/// [`REFRESH_AHEAD`](session::REFRESH_AHEAD), refreshing it first when it is
/// not.
///
/// The refresh runs in a task of its own, so a caller dropped mid-flight
/// never loses a token rotation; the new session is stored before the signal
/// changes. It holds the browser-wide [`CrossTabLock`], so tabs take turns:
/// the store keeps no access token, so every tab refreshes as it loads, and
/// each one in turn sends the cookie the one before it rotated.
///
/// # Errors
///
/// `Unauthorized` when there is no session or its refresh token has expired,
/// which also clears it. A refresh the server rejects (401, 403, 404) clears
/// the session and returns that error; any other failure keeps the session
/// for the next try and returns the error.
pub async fn ensure_fresh(
    mut session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
) -> Result<Session, ClientError> {
    match check(session) {
        Checked::Ready(current) => return Ok(*current),
        Checked::Failed(e) => return Err(e),
        Checked::Stale => {}
    }

    let guard = refresh_lock().lock_owned().await;
    let current = match check(session) {
        Checked::Ready(current) => return Ok(*current),
        Checked::Failed(e) => return Err(e),
        Checked::Stale => session.peek().clone(),
    };
    if current.is_none() {
        return Err(not_logged_in());
    }

    let (tx, rx) = oneshot::channel();
    dioxus::core::spawn_forever(async move {
        let _guard = guard;
        let lock = CrossTabLock::acquire().await;
        // The refresh token rides in the HttpOnly cookie, which the browser
        // sends on its own.
        let http = client.peek().clone();
        let result = match http.refresh_cookie().await {
            Ok(new) => {
                SessionStore::browser().infallible_save(&new);
                session.set(Some(new.clone()));
                tracing::info!("refreshed session");
                Ok(new)
            }
            Err(e) => {
                if e.status().is_some_and(session::refresh_rejected) {
                    tracing::warn!("refresh rejected; clearing session: {e}");
                    SessionStore::browser().infallible_delete();
                    session.set(None);
                } else {
                    tracing::warn!("refresh failed; keeping session: {e}");
                }
                Err(e)
            }
        };
        lock.release();
        let _ = tx.send(result);
    });

    rx.await.unwrap_or_else(|_| {
        Err(ClientError::InternalServerError(
            "refresh task dropped".to_string(),
        ))
    })
}

/// What the stored session allows before any request.
enum Checked {
    /// Good to use as it is.
    Ready(Box<Session>),
    /// Missing or expired, and cleared.
    Failed(ClientError),
    /// Needs a refresh.
    Stale,
}

/// Reads the session without subscribing the caller and clears an expired
/// one.
fn check(mut session: Signal<Option<Session>>) -> Checked {
    let Some(current) = session.peek().clone() else {
        return Checked::Failed(not_logged_in());
    };
    match session::freshness(&current, Utc::now()) {
        Freshness::Expired => {
            tracing::info!("session expired");
            SessionStore::browser().infallible_delete();
            session.set(None);
            Checked::Failed(ClientError::Unauthorized("session expired".to_string()))
        }
        Freshness::Fresh => Checked::Ready(Box::new(current)),
        Freshness::Stale => Checked::Stale,
    }
}

fn not_logged_in() -> ClientError {
    ClientError::Unauthorized("not logged in".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_auth_rejections_drop_the_session() {
        let rejected = |error: ClientError| error.status().is_some_and(session::refresh_rejected);
        let says = || "no".to_string();
        assert!(rejected(ClientError::Unauthorized(says())));
        assert!(rejected(ClientError::Forbidden(says())));
        assert!(rejected(ClientError::NotFound(says())));
        assert!(!rejected(ClientError::InternalServerError(says())));
        assert!(!rejected(ClientError::TooManyRequests(says())));
        assert!(!rejected(ClientError::BadRequest(says())));
    }
}
