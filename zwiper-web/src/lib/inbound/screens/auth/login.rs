//! Login: the hero beside the form on a wide window, stacked below the
//! breakpoint.

use components::{ActionBar, Button, ButtonVariant, TOAST_NORMAL, TextInput, ThemeConfig};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::preferences::PipStyle;
use zwipe_core::http::contracts::auth::HttpAuthenticateUser;

use crate::domain::login::{self, SubmitGate};
use crate::inbound::router::Route;
use crate::inbound::screens::auth::hero::HomeHero;
use crate::inbound::shell::ScreenFrame;
use crate::outbound::session_store::SessionStore;

/// An identifier for the login form to take once, set by Register on
/// success. Provided at the root, so it works while Login sits under
/// Register in the history.
#[derive(Debug, Clone, Copy)]
pub struct LoginPrefill(pub Signal<Option<String>>);

/// The login screen. Enter in either field runs `Log in`.
#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut pip_style: Signal<PipStyle> = use_context();
    let mut theme: Signal<ThemeConfig> = use_context();
    let toast = use_toast();

    let mut identifier = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut is_loading = use_signal(|| false);

    // Takes the prefill whenever one is set: the identifier fills in, the
    // password clears, and the prefill is used up.
    let LoginPrefill(mut prefill) = use_context();
    use_effect(move || {
        if let Some(value) = prefill() {
            identifier.set(value);
            password.set(String::new());
            prefill.set(None);
        }
    });

    let mut attempt_submit = move || {
        match login::submit_gate(*is_loading.peek(), &identifier.peek(), &password.peek()) {
            SubmitGate::Busy => return,
            SubmitGate::MissingInputs => {
                toast.error(
                    login::MISSING_INPUTS.to_string(),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
                return;
            }
            SubmitGate::Send => {}
        }
        is_loading.set(true);
        let request = HttpAuthenticateUser::new(&identifier.peek(), &password.peek());
        spawn(async move {
            let http = client.peek().clone();
            match http.authenticate_user(request).await {
                Ok(new_session) => {
                    SessionStore::browser().infallible_save(&new_session);
                    theme.set(components::theme_from_preferences(&new_session.preferences));
                    pip_style.set(new_session.preferences.pip_style);
                    session.set(Some(new_session));
                    password.set(String::new());
                    is_loading.set(false);
                    navigator.push(Route::Home {});
                }
                Err(e) => {
                    tracing::warn!("login failed: {e}");
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        ScreenFrame { title: "Login", help: true,
            div { class: "screen-content centered content-enter",
                div { class: "web-split",
                    HomeHero {}
                    div { class: "container-sm text-center",
                        form {
                            class: "flex-col",
                            onsubmit: move |event| {
                                event.prevent_default();
                                attempt_submit();
                            },
                            TextInput {
                                value: identifier,
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
                            // Enter in a field submits the form through this.
                            button { r#type: "submit", hidden: true }
                        }
                    }
                }
            }
            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    disabled: is_loading(),
                    onclick: move |_| attempt_submit(),
                    if is_loading() { "Logging in..." } else { "Log in" }
                }
                Button {
                    variant: ButtonVariant::Action,
                    disabled: is_loading(),
                    onclick: move |_| {
                        navigator.push(Route::Register {});
                    },
                    "Create profile"
                }
                Button {
                    variant: ButtonVariant::Action,
                    disabled: is_loading(),
                    onclick: move |_| {
                        navigator.push(Route::ForgotPassword {});
                    },
                    "Forgot password"
                }
            }
        }
    }
}
