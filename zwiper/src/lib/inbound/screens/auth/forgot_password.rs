//! Forgot password screen.

use crate::{
    inbound::components::{fields::text_input::TextInput, screen_header::ScreenHeader},
    outbound::client::{ZwipeClient, auth::forgot_password::ClientForgotPassword},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::{
    domain::{Email, logo},
    http::contracts::auth::HttpRequestPasswordReset,
};

/// Forgot password screen for initiating a password reset.
#[component]
pub fn ForgotPassword() -> Element {
    let navigator = use_navigator();
    let auth_client: Signal<ZwipeClient> = use_context();
    let logo = logo::ZWIPE;

    let email = use_signal(String::new);
    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut email_touched = use_signal(|| false);
    let mut submission_success = use_signal(|| false);
    let toast = use_toast();

    let mut validate_email = move || {
        if Email::new(email()).is_err() {
            email_error.set(Some("Please enter a valid email address".to_string()));
        } else {
            email_error.set(None);
        }
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        validate_email();
        if email_error().is_none() {
            is_loading.set(true);
            let request = HttpRequestPasswordReset::new(&email());
            spawn(async move {
                match auth_client().request_password_reset(request).await {
                    Ok(()) => submission_success.set(true),
                    Err(e) => {
                        tracing::warn!("password reset request failed: {e}");
                        toast.error(
                            e.to_user_message(),
                            ToastOptions::default().duration(Duration::from_millis(3000)),
                        );
                    }
                }
                is_loading.set(false);
            });
        }
    };

    use_effect(move || {
        let value = email();
        if !value.is_empty() && !email_touched() {
            email_touched.set(true);
        }
        if email_touched() || submit_attempted() {
            validate_email();
        }
    });

    rsx! {
        div { class: "screen",
            ScreenHeader { title: "Reset password" }
            div { class: "screen-content centered content-enter",
                div { class: "logo", "{logo}" }
                div { class: "container-sm text-center",
                    if submission_success() {
                        div { class: "message-success",
                            "If that email is registered, a reset link is on its way."
                        }
                    } else {
                        form { class: "flex-col",
                            TextInput {
                                value: email,
                                id: "email",
                                label: "Email",
                                placeholder: "Email",
                                error: email_error(),
                            }
                            if is_loading() {
                                div { class: "spinner" }
                            }
                        }
                    }
                }
            }
            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| navigator.go_back(),
                    "Back to login"
                }
                if !submission_success() {
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| attempt_submit(),
                        "Send reset link"
                    }
                }
            }
        }
    }
}
