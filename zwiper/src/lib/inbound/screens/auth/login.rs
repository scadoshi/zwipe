//! User login screen.

use crate::{
    inbound::{components::fields::text_input::TextInput, router::Router},
    outbound::{
        client::{auth::login::ClientLogin, ZwipeClient},
        session::Persist,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use email_address::EmailAddress;
use std::str::FromStr;
use std::time::Duration;
use zwipe::{
    domain::{
        auth::models::{password::Password, session::Session},
        logo,
        user::models::username::Username,
    },
    inbound::http::handlers::auth::authenticate_user::HttpAuthenticateUser,
};

/// Login form screen for user authentication.
#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let logo = logo::ZWIPE;

    let username_or_email = use_signal(String::new);
    let password = use_signal(String::new);

    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);
    let toast = use_toast();

    let inputs_are_valid = move || {
        (Username::new(username_or_email()).is_ok()
            || EmailAddress::from_str(&username_or_email()).is_ok())
            && Password::new(password()).is_ok()
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        is_loading.set(true);
        if inputs_are_valid() {
            let request = HttpAuthenticateUser::new(&username_or_email(), &password());
            spawn(async move {
                match auth_client().authenticate_user(request).await {
                    Ok(new_session) => {
                        new_session.infallible_save();
                        if new_session.user.email_verified_at.is_none() {
                            toast.info(
                                "verify your email to enable password recovery".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(5000)),
                            );
                        }
                        session.set(Some(new_session));
                        navigator.push(Router::Home {});
                    }
                    Err(e) => {
                        tracing::warn!("login failed: {e}");
                        toast.error(
                            e.to_user_message(),
                            ToastOptions::default().duration(Duration::from_millis(3000)),
                        );
                    }
                }
            });
        } else {
            toast.error(
                "invalid credentials".to_string(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
        is_loading.set(false);
    };

    rsx! {
        div { class: "screen",
            div { class: "screen-content centered content-enter",
            div { class: "logo",  "{logo}" }
            div { class : "container-sm text-center",
                form { class: "flex-col",
                    TextInput {
                        value: username_or_email,
                        id: "identity",
                        label: "username or email",
                        placeholder: "username or email",
                    }
                    TextInput {
                        value: password,
                        id: "password",
                        label: "password",
                        placeholder: "password",
                        input_type: "password",
                    }
                    if is_loading() {
                        div { class : "spinner" }
                    }
                }
            }
        }
        div {
            class: "util-bar",
            button {
                class: "util-btn",
                onclick: move |_| attempt_submit(),
                "login"
            },
            button {
                class : "util-btn",
                onclick: move |_| {
                navigator.push(Router::Register {});
                },
                "create profile"
            }
            button {
                class: "util-btn",
                onclick: move |_| { navigator.push(Router::ForgotPassword {}); },
                "forgot password"
            }
        }
    }
    }
}
