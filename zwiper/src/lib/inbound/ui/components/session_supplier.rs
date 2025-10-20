use crate::domain::development::Spoof;
use crate::outbound::client::auth::session::ActiveSession;
use crate::outbound::{client::auth::AuthClient, session::Persist};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::interval;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn SessionSupplier(children: Element) -> Element {
    let mut session: Signal<Option<Session>> = use_signal(|| {
        Some(Session::spoof())
        // Session::infallible_load()
    });
    use_context_provider(|| session);

    let auth_client: Signal<AuthClient> = use_signal(|| AuthClient::new());
    use_context_provider(|| auth_client);

    spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            tracing::debug!("ensuring active session");
            interval.tick().await;
            let Some(s) = session.read().clone() else {
                continue;
            };
            session.set(auth_client.read().infallible_get_active_session(&s).await);
        }
    });

    rsx! { { children } }
}
