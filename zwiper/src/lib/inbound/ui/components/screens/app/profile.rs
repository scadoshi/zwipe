pub mod change_email;
pub mod change_password;
pub mod change_username;

use crate::{
    inbound::ui::{
        components::{
            auth::bouncer::Bouncer,
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
            screens::app::signal_logout::SignalLogout,
        },
        router::Router,
    },
    outbound::client::ZwipeClient,
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn Profile() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let navigator = use_navigator();

    rsx! {
        Bouncer {
            if let Some(sesh) = session().as_ref() {
                Swipeable { state: swipe_state, config: swipe_config,
                    div { class : "profile-container",
                        h2 { "profile" }

                        div { class : "profile-field",
                            div { class : "profile-field-content",
                                label { "username" }
                                p { { sesh.user.username.to_string() } }
                            }
                            button {
                                class: "profile-field-button",
                                onclick : move |_| {
                                    navigator.push(Router::ChangeUsername {});
                                },
                                "change"
                            }
                        }

                        div { class : "profile-field",
                            div { class : "profile-field-content",
                                label { "email" }
                                p { { sesh.user.email.to_string() } }
                            }
                            button {
                                class: "profile-field-button",
                                onclick : move |_| {
                                    navigator.push(Router::ChangeEmail {});
                                },
                                "change"
                            }
                        }

                        div { class : "profile-field",
                            div { class : "profile-field-content",
                                label { "password" }
                                p { "•••••••" }
                            }
                            button {
                                class: "profile-field-button",
                                onclick : move |_| {
                                    navigator.push(Router::ChangePassword {});
                                },
                                "change"
                            }
                        }

                        button {
                            class: "logout-button",
                            onclick : move |_| session.logout(auth_client),
                            "logout"
                        }

                        button {
                            onclick : move |_| {
                                navigator.push(Router::Home {});
                            },
                            "back"
                        }
                    }
                }
            }
        }
    }
}
