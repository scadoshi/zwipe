//! Register: the hero beside the four fields on a wide window, stacked below
//! the breakpoint.

use components::{ActionBar, Button, ButtonVariant, TOAST_NORMAL, TextInput};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::http::contracts::auth::HttpRegisterUser;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;

use crate::domain::validation;
use crate::inbound::chrome::hint_topic::{self, HintTopic};
use crate::inbound::router::Route;
use crate::inbound::screens::auth::hero::HomeHero;
use crate::inbound::screens::auth::login::LoginPrefill;
use crate::inbound::session::funnel;
use crate::inbound::shell::ScreenFrame;

/// The toast after a registration the server accepted.
const CHECK_INBOX: &str = "Check your inbox to verify your email, then log in";

/// Create profile. Each field starts showing its error line once it has
/// held text, or after a submit attempt, and rechecks on every change. Enter
/// in any field runs `Create profile`.
#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();
    let LoginPrefill(mut login_prefill) = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    use_hook(|| funnel::record(client, AnonymousEventKind::RegisterViewed));

    let username = use_signal(String::new);
    let email = use_signal(String::new);
    let password = use_signal(String::new);
    let confirm_password = use_signal(String::new);
    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    let mut username_touched = use_signal(|| false);
    let mut email_touched = use_signal(|| false);
    let mut password_touched = use_signal(|| false);
    use_effect(move || {
        if !username().is_empty() {
            username_touched.set(true);
        }
    });
    use_effect(move || {
        if !email().is_empty() {
            email_touched.set(true);
        }
    });
    use_effect(move || {
        if !password().is_empty() || !confirm_password().is_empty() {
            password_touched.set(true);
        }
    });

    let attempted = submit_attempted();
    let username_error = (username_touched() || attempted)
        .then(|| validation::username_error(&username()))
        .flatten();
    let email_error = (email_touched() || attempted)
        .then(|| validation::email_error(email().trim()))
        .flatten();
    let password_error = (password_touched() || attempted)
        .then(|| validation::password_error(&password(), &confirm_password(), attempted))
        .flatten();

    let back_to_login = move || {
        if navigator.can_go_back() {
            navigator.go_back();
        } else {
            navigator.replace(Route::Login {});
        }
    };

    let mut attempt_submit = move || {
        if *is_loading.peek() {
            return;
        }
        submit_attempted.set(true);
        let username = username.peek().clone();
        let email = email.peek().trim().to_string();
        let password = password.peek().clone();
        let invalid = validation::username_error(&username).is_some()
            || validation::email_error(&email).is_some()
            || validation::password_error(&password, &confirm_password.peek(), true).is_some();
        if invalid {
            return;
        }
        is_loading.set(true);
        funnel::record(client, AnonymousEventKind::RegisterSubmitted);
        let request = HttpRegisterUser::new(&username, &email, &password);
        spawn(async move {
            let http = client.peek().clone();
            // The same answer whether or not the email already has an
            // account, so the screen says the same either way.
            match http.register(request).await {
                Ok(_) => {
                    is_loading.set(false);
                    login_prefill.set(Some(username));
                    toast.info(
                        CHECK_INBOX.to_string(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                    back_to_login();
                }
                Err(e) => {
                    tracing::warn!("register failed: {e}");
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
        ScreenFrame { title: "Create profile", help: true,
            div { class: "screen-content content-enter",
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
                                value: username,
                                id: "username",
                                label: "Username",
                                placeholder: "Username",
                                error: username_error,
                            }
                            TextInput {
                                value: email,
                                id: "email",
                                label: "Email",
                                placeholder: "Email",
                                input_type: "email",
                                error: email_error,
                            }
                            TextInput {
                                value: password,
                                id: "password",
                                label: "Password",
                                placeholder: "Password",
                                input_type: "password",
                                error: password_error,
                                on_info: hint_topic::open_topic(HintTopic::Password),
                            }
                            TextInput {
                                value: confirm_password,
                                id: "confirm_password",
                                label: "Confirm password",
                                placeholder: "Confirm password",
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
                    onclick: move |_| back_to_login(),
                    "Back to login"
                }
                Button {
                    variant: ButtonVariant::Action,
                    disabled: is_loading(),
                    onclick: move |_| attempt_submit(),
                    if is_loading() { "Creating..." } else { "Create profile" }
                }
            }
        }
    }
}
