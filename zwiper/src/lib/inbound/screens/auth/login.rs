//! User login screen.

use crate::{
    inbound::{components::fields::text_input::TextInput, router::Router},
    outbound::{
        client::{auth::login::ClientLogin, ZwipeClient},
        session::Persist,
    },
};
use zwipe_core::domain::user::models::theme::ThemeConfig;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe::domain::auth::models::password::Password;
use zwipe_core::http::contracts::auth::HttpAuthenticateUser;
use zwipe_core::domain::{
    auth::models::session::Session,
    logo,
    user::username::Username,
    Email,
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
            || Email::new(username_or_email()).is_ok())
            && Password::new(password()).is_ok()
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        if !inputs_are_valid() {
            toast.error(
                "Invalid credentials".to_string(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
            return;
        }
        is_loading.set(true);
        let request = HttpAuthenticateUser::new(&username_or_email(), &password());
        spawn(async move {
            match auth_client().authenticate_user(request).await {
                Ok(new_session) => {
                    new_session.infallible_save();
                    // Apply theme from preferences
                    let mut theme: Signal<ThemeConfig> = use_context();
                    theme.set(ThemeConfig::from(&new_session.preferences));
                    session.set(Some(new_session));
                    is_loading.set(false);
                    navigator.push(Router::Home {});
                }
                Err(e) => {
                    tracing::warn!("login failed: {e}");
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "screen",
            div { class: "page-header", h2 { "Login" } }
            div { class: "screen-content centered content-enter",
            div { class: "logo",  "{logo}" }
            div { class : "container-sm text-center",
                form { class: "flex-col",
                    TextInput {
                        value: username_or_email,
                        id: "identity",
                        label: "Username or email",
                        placeholder: "Username or email",
                    }
                    TextInput {
                        value: password,
                        id: "password",
                        label: "Password",
                        placeholder: "Password",
                        input_type: "password",
                    }
                }
            }
        }
        div {
            class: "util-bar",
            button {
                class: "util-btn",
                disabled: is_loading(),
                onclick: move |_| attempt_submit(),
                if is_loading() { "Logging in..." } else { "Log in" }
            },
            button {
                class : "util-btn",
                disabled: is_loading(),
                onclick: move |_| {
                navigator.push(Router::Register {});
                },
                "Create profile"
            }
            button {
                class: "util-btn",
                disabled: is_loading(),
                onclick: move |_| { navigator.push(Router::ForgotPassword {}); },
                "Forgot password"
            }
        }
    }
    }
}
