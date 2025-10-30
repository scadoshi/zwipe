use crate::outbound::{
    client::{auth::refresh::ClientRefresh, ZwipeClient},
    session::Persist,
};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::interval;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::handlers::auth::refresh_session::HttpRefreshSession,
};

pub trait Upkeep {
    fn upkeep(self, client: Signal<ZwipeClient>);
}

impl Upkeep for Signal<Option<Session>> {
    fn upkeep(self, client: Signal<ZwipeClient>) {
        let mut session = self;

        spawn(async move {
            let Some(current) = session() else {
                tracing::debug!("session is none");
                return;
            };

            if current.is_expired() {
                tracing::info!("session has expired");
                session.set(None);
                return;
            }

            if current.access_token.is_expired() {
                let request = HttpRefreshSession::from(&current);
                match client().refresh(&request).await {
                    Ok(new) => {
                        session.set(Some(new));
                        tracing::info!("refreshed session");
                    }
                    Err(e) => {
                        session.set(None);
                        tracing::error!("error refreshing session {e}");
                    }
                }
            }
            tracing::info!("session still active");
        });
    }
}

pub fn spawn_upkeeper() {
    tracing::debug!("upkeeper spawned");
    let session = use_signal(|| Session::infallible_load());
    use_context_provider(|| session);

    let client = use_signal(|| ZwipeClient::new());
    use_context_provider(|| client);

    spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            session.upkeep(client);
        }
    });
}
