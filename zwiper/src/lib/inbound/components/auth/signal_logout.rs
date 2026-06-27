//! Logout functionality for session signals.
//!
//! Provides a trait extension for session signals to perform logout operations,
//! including server-side session invalidation and local storage cleanup.

use crate::{
    inbound::components::auth::ensure_session::EnsureFresh,
    outbound::{
        client::{ZwipeClient, auth::logout::ClientLogout},
        session::Persist,
    },
};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

/// Trait for session signals that can perform logout operations.
pub trait SignalLogout {
    /// Logs out the current user, invalidating the session on the server.
    fn logout(self, auth_client: Signal<ZwipeClient>);
}

impl SignalLogout for Signal<Option<Session>> {
    fn logout(self, auth_client: Signal<ZwipeClient>) {
        let mut session = self;

        spawn(async move {
            let current = match session.ensure_fresh(auth_client).await {
                Ok(current) => current,
                Err(e) => {
                    // ensure_fresh already cleared signal + keyring on auth
                    // rejection; on transient errors still honor the user's
                    // logout locally — the server token expires naturally.
                    tracing::warn!("logout without server invalidation: {e}");
                    if let Some(current) = session.peek().clone() {
                        current.infallible_delete();
                    }
                    session.set(None);
                    return;
                }
            };

            // Always clear the local session — the user asked to log out regardless
            // of whether the server can be reached to invalidate the refresh token.
            current.infallible_delete();
            session.set(None);

            if let Err(e) = auth_client().logout(&current).await {
                tracing::warn!("server-side logout failed (token will expire naturally): {e}");
            }
        });
    }
}
