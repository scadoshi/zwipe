//! Logging out: tell the server, then forget the session whatever it says.

use std::time::Duration;

use dioxus::prelude::*;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::outbound::session_store::SessionStore;
use crate::outbound::timer;

/// How long logout waits on the server before clearing the session anyway.
pub const LOGOUT_TIMEOUT: Duration = Duration::from_secs(5);

/// Asks the server to end the session, then clears it here whether the
/// server answered, failed or took longer than [`LOGOUT_TIMEOUT`]. The auth
/// gate then shows Login. No toast either way.
pub fn logout(session: Signal<Option<Session>>, client: Signal<ZwipeClient>) {
    spawn(async move {
        match timer::timeout(LOGOUT_TIMEOUT, invalidate_on_server(session, client)).await {
            Some(Ok(())) => {}
            Some(Err(e)) => tracing::warn!("server-side logout failed: {e}"),
            None => tracing::warn!("server-side logout timed out after {LOGOUT_TIMEOUT:?}"),
        }
        clear_local(session);
    });
}

/// Refreshes the session if it needs it, then sends the logout request.
async fn invalidate_on_server(
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
) -> Result<(), ClientError> {
    let current = ensure_fresh(session, client).await?;
    let http = client.peek().clone();
    http.logout(&current).await
}

/// Drops the stored session and the signal.
fn clear_local(mut session: Signal<Option<Session>>) {
    SessionStore::browser().infallible_delete();
    session.set(None);
}
