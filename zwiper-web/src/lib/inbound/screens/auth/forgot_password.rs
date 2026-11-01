//! Forgot password: the email field as its own panel, centered at the top
//! of zite's page, with `Send reset link` and `Log in` stacked under the
//! field on a wide window and in the bar below the breakpoint, and the
//! `Check your email` panel with `Log in` in its place once sent.
//!
//! The reset itself happens on the website, from the emailed link.

use components::{ActionBar, Button, ButtonVariant, Panel, TOAST_NORMAL, TextInput};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::http::contracts::auth::HttpRequestPasswordReset;

use crate::domain::validation;
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;

/// Reset password: asks for an email, then says a link is on its way. The
/// answer never says whether the address has an account. Neither button
/// waits on the request, so a second `Send reset link` sends again. Enter in
/// the field runs `Send reset link`.
#[component]
pub fn ForgotPassword() -> Element {
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

    let submit = use_callback(move |()| attempt_submit());

    rsx! {
        ScreenFrame { title: "Reset password",
            div { class: "screen-content centered content-enter",
                div { class: "web-auth web-page",
                    if sent() {
                        div { class: "outcome-success web-outcome-panel",
                            Panel { title: "Check your email", centered: true,
                                p { "If that email is registered, a reset link is on its way." }
                                div { class: "web-form-actions web-form-actions-stack",
                                    ForgotActions { sent: true, on_submit: submit }
                                }
                            }
                        }
                    } else {
                        div { class: "container-sm text-center web-form-panel",
                            form {
                                class: "flex-col",
                                onsubmit: move |event| {
                                    event.prevent_default();
                                    submit.call(());
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
                            div { class: "web-form-actions web-form-actions-stack",
                                ForgotActions { sent: false, on_submit: submit }
                            }
                        }
                    }
                }
            }
            ActionBar { class: "web-form-toolbar",
                ForgotActions { sent: sent(), on_submit: submit }
            }
        }
    }
}

/// `Send reset link` until the link is sent, then `Log in`. Stacked under
/// the form (or the sent panel's line) on a wide window, in the action bar
/// below the breakpoint.
#[component]
fn ForgotActions(sent: bool, on_submit: Callback<()>) -> Element {
    let navigator = use_navigator();
    rsx! {
        if !sent {
            Button {
                variant: ButtonVariant::Action,
                onclick: move |_| on_submit.call(()),
                "Send reset link"
            }
        }
        Button {
            variant: ButtonVariant::Action,
            onclick: move |_| {
                navigator.push(Route::Login {});
            },
            "Log in"
        }
    }
}
