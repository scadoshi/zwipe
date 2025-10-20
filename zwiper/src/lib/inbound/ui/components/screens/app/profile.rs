pub mod change_email;
pub mod change_password;
pub mod change_username;

use crate::{inbound::ui::router::Router, outbound::session::Persist};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

#[component]
pub fn Profile() -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let navigator = use_navigator();

    rsx! {
        if let Some(user_session) = session.read().as_ref() {
            div { class : "nicely-centered",
                div { class : "profile-container",
                    h2 { "profile" }

                    div { class : "profile-field",
                        div { class : "profile-field-content",
                            label { "username" }
                            p { { user_session.user.username.to_string() } }
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
                            p { { user_session.user.email.to_string() } }
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
                        onclick : move |_| {
                            navigator.push(Router::Home {});
                        },
                        "back"
                    }

                    button {
                        class: "logout-button",
                        onclick : move |_| {
                            // todo: Call logout endpoint on server to invalidate refresh token
                            if let Some(current_session) = session.read().clone() {
                                let _ = current_session.delete();
                            }
                            session.set(None);
                            navigator.push(Router::Login {});
                        },
                        "logout"
                    }
                }
            }
        }
    }
}
