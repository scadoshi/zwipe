pub mod change_email;
pub mod change_password;
pub mod change_username;

use crate::{
    inbound::ui::{
        components::{
            auth::bouncer::Bouncer,
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
        router::Router,
        screens::app::signal_logout::SignalLogout,
    },
    outbound::client::ZwipeClient,
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn Profile() -> Element {
    let swipe_state = use_signal(SwipeState::new);
    let swipe_config = SwipeConfig::blank();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut confirm_logout = use_signal(|| false);

    let navigator = use_navigator();

    rsx! {
        Bouncer {
            if let Some(sesh) = session().as_ref() {
                Swipeable { state: swipe_state, config: swipe_config,
                    div { class : "container-sm",
                        h2 { class: "text-center mb-4 font-light tracking-wider", "profile" }

                        div { class : "flex items-center flex-between mb-4 gap-2",
                            div { class : "flex-1",
                                label { class: "label", "username" }
                                p { class: "text-base font-light mb-1",
                                    { sesh.user.username.to_string() }
                                }
                            }
                            button { class: "btn btn-sm",
                                onclick : move |_| {
                                    navigator.push(Router::ChangeUsername {});
                                },
                                "change"
                            }
                        }

                        div { class : "flex items-center flex-between mb-4 gap-2",
                            div { class : "flex-1",
                                label { class: "label", "email" }
                                p { class: "text-base font-light mb-1",
                                    { sesh.user.email.to_string() }
                                }
                            }
                            button { class: "btn btn-sm",
                                onclick : move |_| {
                                    navigator.push(Router::ChangeEmail {});
                                },
                                "change"
                            }
                        }

                        div { class : "flex items-center flex-between mb-4 gap-2",
                            div { class : "flex-1",
                                label { class: "label", "password" }
                                p { class: "text-base font-light mb-1", "•••••••" }
                            }
                            button { class: "btn btn-sm",
                                onclick : move |_| {
                                    navigator.push(Router::ChangePassword {});
                                },
                                "change"
                            }
                        }

                        if !confirm_logout() {
                            button { class: "btn",
                                onclick : move |_| confirm_logout.set(true),
                                "logout"
                            }
                        }

                        if confirm_logout() {
                            label { class: "text-center label", r#for : "confirmation-prompt", "are you sure?" }
                            div { class : "flex flex-between gap-2",
                                id : "confirmation-prompt",
                                button { class : "btn btn-half",
                                    onclick: move |_| session.logout(client),
                                    "yes"
                                }
                                button { class : "btn btn-half",
                                    onclick: move |_| confirm_logout.set(false),
                                    "no"
                                }
                            }
                        }

                        button { class: "btn",
                            onclick : move |_| {
                                navigator.go_back();
                            },
                            "back"
                        }
                    }
                }
            }
        }
    }
}
