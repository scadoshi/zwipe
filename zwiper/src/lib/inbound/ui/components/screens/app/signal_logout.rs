use crate::{
    inbound::ui::components::auth::session_upkeep::Upkeep,
    outbound::{
        client::auth::{logout::AuthClientLogout, AuthClient},
        session::Persist,
    },
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

pub trait SignalLogout {
    fn logout(self, auth_client: Signal<AuthClient>);
}

impl SignalLogout for Signal<Option<Session>> {
    fn logout(self, auth_client: Signal<AuthClient>) {
        let mut session = self;

        spawn(async move {
            session.upkeep(auth_client);
            let Some(current) = session.read().clone() else {
                return;
            };

            match auth_client.read().logout(&current).await {
                Ok(()) => {
                    current.infallible_delete();
                    session.set(None);
                }
                Err(e) => tracing::error!("failed to logout: {e}"),
            }
        });
    }
}
