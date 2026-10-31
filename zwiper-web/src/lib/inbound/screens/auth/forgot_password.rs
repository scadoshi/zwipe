//! Forgot password: the hero beside the email field on a wide window,
//! stacked below the breakpoint.
//!
//! The reset itself happens on the website, from the emailed link.

use components::{ActionBar, Button, ButtonVariant, Panel, TOAST_NORMAL, TextInput};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::http::contracts::auth::HttpRequestPasswordReset;

use crate::domain::validation;
use crate::inbound::router::Route;
use crate::inbound::screens::auth::hero::HomeHero;
use crate::inbound::shell::ScreenFrame;

/// Reset password: asks for an email, then says a link is on its way. The
/// answer never says whether the address has an account. Neither button
/// waits on the request, so a second `Send reset link` sends again. Enter in
/// the field runs `Send reset link`.
#[component]
pub fn ForgotPassword() -> Element {
    let navigator = use_navigator();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    let email = use_signal(String::new);
    let mut submit_attempted = use_signal(|| false);
    let mut email_touched = use_signal(|| false);
    let mut is_loading = use_signal(|| false);
    let mut sent = use_signal(|| false);
    use_effect(move || {
        if !email().is_empty() {
            email_touched.set(true);
        }
    });
    let email_error = (email_touched() || submit_attempted())
        .then(|| validation::reset_email_error(&email()))
        .flatten();

    let back_to_login = move |_| {
        if navigator.can_go_back() {
            navigator.go_back();
        } else {
            navigator.replace(Route::Login {});
        }
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        let typed = email.peek().clone();
        if validation::reset_email_error(&typed).is_some() {
            return;
        }
        is_loading.set(true);
        let request = HttpRequestPasswordReset::new(&typed);
        spawn(async move {
            let http = client.peek().clone();
            match http.request_password_reset(request).await {
                Ok(()) => sent.set(true),
                Err(e) => {
                    tracing::warn!("password reset request failed: {e}");
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                }
            }
            is_loading.set(false);
        });
    };

    rsx! {
        ScreenFrame { title: "Reset password", help: true,
            div { class: "screen-content centered content-enter",
                div { class: "web-split",
                    HomeHero {}
                    if sent() {
                        div { class: "outcome-success",
                            Panel { title: "Check your email", centered: true,
                                p { "If that email is registered, a reset link is on its way." }
                            }
                        }
                    } else {
                        div { class: "container-sm text-center",
                            form {
                                class: "flex-col",
                                onsubmit: move |event| {
                                    event.prevent_default();
                                    attempt_submit();
                                },
                                TextInput {
                                    value: email,
                                    id: "email",
                                    label: "Email",
                                    placeholder: "Email",
                                    error: email_error,
                                }
                                if is_loading() {
                                    div { class: "spinner" }
                                }
                                button { r#type: "submit", hidden: true }
                            }
                        }
                    }
                }
            }
            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: back_to_login,
                    "Back to login"
                }
                if !sent() {
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| attempt_submit(),
                        "Send reset link"
                    }
                }
            }
        }
    }
}
