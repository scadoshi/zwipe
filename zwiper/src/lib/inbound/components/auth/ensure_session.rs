//! Awaitable session freshness guard with single-flight refresh.
//!
//! Replaces the old fire-and-forget `Upkeep` pattern. Call sites await
//! `ensure_fresh` to obtain a session whose access token is valid; if a
//! refresh is needed, exactly one `POST /api/auth/refresh` goes out no
//! matter how many callers race (cold start mounts several resources at
//! once). The rotated session is persisted to the OS keyring before the
//! signal updates, so a force-quit can never strand a dead refresh token.

use std::sync::{Arc, OnceLock};

use crate::outbound::{
    client::{ZwipeClient, auth::refresh::ClientRefresh},
    session::Persist,
};
use dioxus::prelude::*;
use tokio::sync::{Mutex, oneshot};
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpRefreshSession;

/// Process-wide single-flight lock — at most one refresh request in flight.
/// Losers of the race wait for the winner's result instead of firing their own.
static REFRESH_LOCK: OnceLock<Arc<Mutex<()>>> = OnceLock::new();

fn refresh_lock() -> Arc<Mutex<()>> {
    Arc::clone(REFRESH_LOCK.get_or_init(|| Arc::new(Mutex::new(()))))
}

/// Trait for session signals that can vouch for access-token freshness.
pub trait EnsureFresh {
    /// Returns a session with a valid access token, refreshing (single-flight)
    /// and persisting if necessary. `Err` means "do not make the authed call":
    /// auth rejections clear the session (the bouncer redirects to login);
    /// transient network/server errors leave the session untouched so the
    /// next attempt can retry.
    fn ensure_fresh(
        self,
        client: Signal<ZwipeClient>,
    ) -> impl std::future::Future<Output = Result<Session, ApiError>>;
}

impl EnsureFresh for Signal<Option<Session>> {
    async fn ensure_fresh(self, client: Signal<ZwipeClient>) -> Result<Session, ApiError> {
        let mut session = self;

        // Fast path — no lock. peek() rather than session() so resources
        // calling this aren't subscribed to re-run on every refresh.
        let Some(current) = session.peek().clone() else {
            return Err(ApiError::Unauthorized("not logged in".to_string()));
        };
        if current.is_expired() {
            tracing::info!("session expired (refresh token dead)");
            current.infallible_delete();
            session.set(None);
            return Err(ApiError::Unauthorized("session expired".to_string()));
        }
        if !current.access_token.is_expired() {
            return Ok(current);
        }

        // Slow path — single flight. Whoever wins the lock refreshes;
        // everyone else blocks here, then passes the re-check below.
        let guard = refresh_lock().lock_owned().await;

        let Some(current) = session.peek().clone() else {
            return Err(ApiError::Unauthorized("not logged in".to_string()));
        };
        if !current.access_token.is_expired() {
            return Ok(current); // a concurrent caller already refreshed
        }

        // Cancellation-proof commit: dioxus drops resource futures freely,
        // and losing a token rotation mid-flight would strand the session
        // once the server enforces single-use. The detached task owns the
        // lock guard and always completes save + signal update; the oneshot
        // carries the result back to whichever caller is still listening.
        let (tx, rx) = oneshot::channel();
        dioxus::core::spawn_forever(async move {
            let _guard = guard;
            let request =
                HttpRefreshSession::new(&current.user.id.to_string(), &current.refresh_token.value);
            // clone the client out before awaiting — holding the signal read
            // guard across the await would block writes while pending
            let client = client.peek().clone();
            let result = match client.refresh(&request).await {
                Ok(new) => {
                    // persist before set — keyring must always hold the
                    // live rotated token
                    new.infallible_save();
                    session.set(Some(new.clone()));
                    tracing::info!("refreshed session");
                    Ok(new)
                }
                Err(e) => {
                    match &e {
                        ApiError::Unauthorized(_)
                        | ApiError::Forbidden(_)
                        | ApiError::NotFound(_) => {
                            tracing::warn!("refresh rejected; clearing session: {e}");
                            current.infallible_delete();
                            session.set(None);
                        }
                        _ => {
                            tracing::warn!("refresh failed transiently; keeping session: {e}");
                        }
                    }
                    Err(e)
                }
            };
            let _ = tx.send(result);
        });

        rx.await
            .unwrap_or_else(|_| Err(ApiError::Network("refresh task dropped".to_string())))
    }
}
