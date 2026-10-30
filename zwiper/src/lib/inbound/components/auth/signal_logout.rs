//! Logout functionality for session signals.
//!
//! Provides a trait extension for session signals to perform logout operations,
//! including server-side session invalidation and local storage cleanup.

use std::fmt::Display;
use std::future::Future;
use std::time::Duration;

use dioxus::prelude::*;
use tokio::time;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::outbound::session::Persist;

/// How long logout waits on the server before clearing the session anyway.
/// Long enough for a slow mobile round trip, short enough that an offline
/// tap still feels like it did something.
pub const LOGOUT_TIMEOUT: Duration = Duration::from_secs(5);

/// Trait for session signals that can perform logout operations.
pub trait SignalLogout {
    /// Logs out the current user: asks the server to invalidate the session,
    /// then clears it locally whether or not the server answered within
    /// [`LOGOUT_TIMEOUT`].
    fn logout(self, auth_client: Signal<ZwipeClient>);
}

impl SignalLogout for Signal<Option<Session>> {
    fn logout(self, auth_client: Signal<ZwipeClient>) {
        let session = self;
        spawn(server_first(
            invalidate_on_server(session, auth_client),
            LOGOUT_TIMEOUT,
            move || clear_local(session),
        ));
    }
}

/// Refreshes the session if it needs it, then sends the logout request.
async fn invalidate_on_server(
    session: Signal<Option<Session>>,
    auth_client: Signal<ZwipeClient>,
) -> Result<(), ClientError> {
    let current = session.ensure_fresh(auth_client).await?;
    let client = auth_client.peek().clone();
    client.logout(&current).await
}

/// Drops the stored session and the signal. `ensure_fresh` may already have
/// done both after an auth rejection, which leaves nothing to delete.
fn clear_local(mut session: Signal<Option<Session>>) {
    if let Some(current) = session.peek().clone() {
        current.infallible_delete();
    }
    session.set(None);
}

/// Runs `server` for up to `timeout`, then `clear`. The user asked to log
/// out, so a failed, slow or unreachable server still ends in `clear`; the
/// server-side token then expires on its own.
async fn server_first<E: Display>(
    server: impl Future<Output = Result<(), E>>,
    timeout: Duration,
    clear: impl FnOnce(),
) {
    match time::timeout(timeout, server).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            tracing::warn!("server-side logout failed (token will expire naturally): {e}");
        }
        Err(_) => tracing::warn!("server-side logout timed out after {timeout:?}"),
    }
    clear();
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::future;
    use std::rc::Rc;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Event {
        LogoutSent,
        LogoutAnswered,
        SessionCleared,
    }

    #[tokio::test]
    async fn the_logout_request_goes_out_before_the_session_clears() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let server_events = Rc::clone(&events);
        let server = async move {
            server_events.borrow_mut().push(Event::LogoutSent);
            tokio::task::yield_now().await;
            server_events.borrow_mut().push(Event::LogoutAnswered);
            Ok::<(), ClientError>(())
        };
        let clear_events = Rc::clone(&events);

        server_first(server, LOGOUT_TIMEOUT, move || {
            clear_events.borrow_mut().push(Event::SessionCleared);
        })
        .await;

        assert_eq!(
            *events.borrow(),
            [
                Event::LogoutSent,
                Event::LogoutAnswered,
                Event::SessionCleared
            ]
        );
    }

    #[tokio::test]
    async fn a_failed_logout_still_clears_the_session() {
        let cleared = Rc::new(RefCell::new(false));
        let flag = Rc::clone(&cleared);
        let server = async { Err(ClientError::Unauthorized("gone".to_string())) };

        server_first(server, LOGOUT_TIMEOUT, move || *flag.borrow_mut() = true).await;

        assert!(
            *cleared.borrow(),
            "a rejected logout left the session in place"
        );
    }

    #[tokio::test]
    async fn a_server_that_never_answers_still_clears_the_session_after_the_timeout() {
        let timeout = Duration::from_millis(50);
        let cleared_at = Rc::new(RefCell::new(None));
        let at = Rc::clone(&cleared_at);
        let started = time::Instant::now();

        server_first(
            future::pending::<Result<(), ClientError>>(),
            timeout,
            move || *at.borrow_mut() = Some(time::Instant::now()),
        )
        .await;

        let cleared_at = cleared_at
            .borrow()
            .expect("the session never cleared while the server hung");
        assert!(
            cleared_at.duration_since(started) >= timeout,
            "cleared after {:?}, before the {timeout:?} timeout",
            cleared_at.duration_since(started)
        );
    }
}
