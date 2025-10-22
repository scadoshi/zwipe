use crate::{
    inbound::ui::{
        components::{
            auth::bouncer::Bouncer,
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
            screens::app::signal_logout::SignalLogout,
        },
        router::Router,
    },
    outbound::client::auth::AuthClient,
};
use dioxus::prelude::*;
use zwipe::domain::{auth::models::session::Session, logo};

#[component]
pub fn Home() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let auth_client: Signal<AuthClient> = use_context();
    let session: Signal<Option<Session>> = use_context();

    let logo = logo::ZWIPE;

    rsx! {
        Bouncer {
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
                        onclick : move |_| session.logout(auth_client),
                        "logout"
                    }
                }
            }
        }
    }
}
