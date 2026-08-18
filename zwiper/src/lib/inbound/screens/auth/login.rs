//! User login screen.

use crate::{
    inbound::{
        components::{fields::text_input::TextInput, screen_header::ScreenHeader},
        router::Router,
    },
    outbound::{
        client::{ZwipeClient, auth::login::ClientLogin},
        session::Persist,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::{
    domain::{auth::models::session::Session, logo, user::models::theme::ThemeConfig},
    http::contracts::auth::HttpAuthenticateUser,
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

    let mut is_loading = use_signal(|| false);
    let toast = use_toast();

    // Login deliberately does NOT apply registration policy to what's typed.
    // Those rules can tighten, and an account whose password or username predates
    // a change would be rejected here and never reach the server to be told why,
    // locking out a user whose credentials are perfectly correct. The server is
    // the only authority on whether credentials are valid; all we check is that
    // there is something to send.
    let inputs_are_present =
        move || !username_or_email().trim().is_empty() && !password().is_empty();

    let mut attempt_submit = move || {
        if !inputs_are_present() {
            toast.error(
                "Enter your username or email and password".to_string(),
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
            ScreenHeader { title: "Login" }
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
        ActionBar {
            Button {
                variant: ButtonVariant::Util,
                disabled: is_loading(),
                onclick: move |_| attempt_submit(),
                if is_loading() { "Logging in..." } else { "Log in" }
            },
            Button {
                variant: ButtonVariant::Util,
                disabled: is_loading(),
                onclick: move |_| {
                navigator.push(Router::Register {});
                },
                "Create profile"
            }
            Button {
                variant: ButtonVariant::Util,
                disabled: is_loading(),
                onclick: move |_| { navigator.push(Router::ForgotPassword {}); },
                "Forgot password"
            }
        }
    }
    }
}
