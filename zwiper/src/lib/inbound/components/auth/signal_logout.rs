use crate::{
    inbound::components::auth::session_upkeep::Upkeep,
    outbound::{
        client::{auth::logout::ClientLogout, ZwipeClient},
        session::Persist,
    },
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

pub trait SignalLogout {
    fn logout(self, auth_client: Signal<ZwipeClient>);
}

impl SignalLogout for Signal<Option<Session>> {
    fn logout(self, auth_client: Signal<ZwipeClient>) {
        let mut session = self;

        spawn(async move {
            session.upkeep(auth_client);
            let Some(current) = session() else {
                return;
            };

            match auth_client().logout(&current).await {
                Ok(()) => {
                    current.infallible_delete();
                    session.set(None);
                }
                Err(e) => tracing::error!("failed to logout: {e}"),
            }
        });
    }
}
