use crate::outbound::{
    client::auth::{logout::AuthClientLogout, session::AuthClientSession, AuthClient},
    session::Persist,
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

pub trait SignalLogout {
    fn logout(self, auth_client: Signal<AuthClient>);
}

impl SignalLogout for Signal<Option<Session>> {
    fn logout(self, auth_client: Signal<AuthClient>) {
        let mut session = self;
        let Some(current) = session.read().clone() else {
            return;
        };

        spawn(async move {
            let Some(active) = auth_client
                .read()
                .infallible_get_active_session(&current)
                .await
            else {
                current.infallible_delete();
                session.set(None);
                return;
            };

            match auth_client.read().logout(&active).await {
                Ok(()) => {
                    active.infallible_delete();
                    session.set(None);
                }
                Err(e) => tracing::error!("failed to logout: {e}"),
            }
        });
    }
}
