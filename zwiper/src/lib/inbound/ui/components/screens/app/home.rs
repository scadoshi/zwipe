use crate::{
    inbound::ui::{
        components::interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        router::Router,
    },
    outbound::{
        client::auth::{logout::Logout, session::ActiveSession, AuthClient},
        session::Persist,
    },
};
use dioxus::prelude::*;
use zwipe::domain::{auth::models::session::Session, logo::logo};

#[component]
pub fn Home() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let logo = logo();

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "logo", "{logo}" }
            div { class : "form-container",
                button {
                    onclick : move |_| {
                        navigator.push(Router::Profile {} );
                    }, "profile"
                }
                button {
                    onclick : move |_| {
                        navigator.push(Router::Decks {} );
                    }, "decks"
                }
                button {
                    onclick : move |_| {
                        spawn(async move {
                            let Some(current) = session.read().clone() else {
                                navigator.push(Router::Login {});
                                return;
                            };

                            let Some(active) = auth_client.read().infallible_get_active_session(&current).await else {
                                navigator.push(Router::Login {});
                                return;
                            };

                            match auth_client.read().logout(&active).await {
                                Ok(()) => {
                                    if let Err(e) = active.delete() {
                                        tracing::error!("failed to delete session from keyring: {e}");
                                    }
                                    session.set(None);
                                    navigator.push(Router::Login {});
                                }
                                Err(e) => tracing::error!("failed to logout: {e}"),
                            }
                        });
                    }, "logout"
                }
            }
        }
    }
}
