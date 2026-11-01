//! Forgot password: on a wide window the hero card at the top of zite's page
//! and the email field under it as its own panel, with the actions under
//! the field, and the `Check your email` panel in its place once sent;
//! below the breakpoint the app's stack with the actions in the bar.
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

    let back_to_login = move || {
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

    let back = use_callback(move |()| back_to_login());
    let submit = use_callback(move |()| attempt_submit());

    rsx! {
        ScreenFrame { title: "Reset password", help: true,
            div { class: "screen-content centered content-enter",
                div { class: "web-auth web-page",
                    div { class: "web-hero", HomeHero {} }
                    if sent() {
                        div { class: "outcome-success web-outcome-panel",
                            Panel { title: "Check your email", centered: true,
                                p { "If that email is registered, a reset link is on its way." }
                                div { class: "web-form-actions",
                                    ForgotActions { sent: true, on_back: back, on_submit: submit }
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
                            div { class: "web-form-actions",
                                ForgotActions { sent: false, on_back: back, on_submit: submit }
                            }
                        }
                    }
                }
            }
            ActionBar { class: "web-form-toolbar",
                ForgotActions { sent: sent(), on_back: back, on_submit: submit }
            }
        }
    }
}

/// `Back to login`, then `Send reset link` until the link is sent. Under
/// the form (or the sent panel's line) on a wide window, in the action bar
/// below the breakpoint.
#[component]
fn ForgotActions(sent: bool, on_back: Callback<()>, on_submit: Callback<()>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Action,
            onclick: move |_| on_back.call(()),
            "Back to login"
        }
        if !sent {
            Button {
                variant: ButtonVariant::Action,
                onclick: move |_| on_submit.call(()),
                "Send reset link"
            }
        }
    }
}
