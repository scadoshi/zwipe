//! Change email screen.

use crate::domain::error::UserFacing;
use crate::inbound::components::auth::{bouncer::Bouncer, session_upkeep::Upkeep};
use crate::inbound::components::fields::text_input::TextInput;
use crate::outbound::client::user::change_email::ClientChangeEmail;
use crate::outbound::client::ZwipeClient;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use email_address::EmailAddress;
use std::str::FromStr;
use std::time::Duration;
use zwipe::domain::auth::models::password::Password;
use zwipe_core::http::contracts::auth::HttpChangeEmail;
use zwipe_core::domain::auth::models::session::Session;

/// Form screen for updating user's email address.
#[component]
pub fn ChangeEmail() -> Element {
    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let mut new_email = use_signal(String::new);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_email = move || {
        if let Err(e) = EmailAddress::from_str(&new_email()) {
            email_error.set(Some(e.to_user_facing_string()));
        } else {
            email_error.set(None)
        }
    };

    let mut password = use_signal(String::new);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_password = move || {
        if Password::new(password()).is_err() {
            password_error.set(Some("Invalid password".to_string()));
        } else {
            password_error.set(None);
        }
    };

    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);
    let toast = use_toast();

    let mut inputs_are_valid = move || {
        validate_email();
        validate_password();
        email_error().is_none() && password_error().is_none()
    };

    let mut clear_inputs = move || {
        new_email.set(String::new());
        password.set(String::new());
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        if inputs_are_valid() {
            tracing::info!("change email to {}", new_email());
            let request = HttpChangeEmail::new(&new_email(), &password());
            is_loading.set(true);
            spawn(async move {
                session.upkeep(auth_client);
                let Some(mut session_value) = session() else {
                    toast.error(
                        "Session expired — please log in again".to_string(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    is_loading.set(false);
                    return;
                };

                match auth_client().change_email(request, &session_value).await {
                    Ok(updated_user) => {
                        let new_email = updated_user.email.clone();
                        session_value.user.email = updated_user.email;
                        session.set(Some(session_value));
                        toast.success(
                            format!("Email changed to {}", new_email),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                        clear_inputs();
                        submit_attempted.set(false);
                        is_loading.set(false);
                    }
                    Err(e) => {
                        tracing::warn!("change email failed: {e}");
                        toast.error(
                            e.to_user_message(),
                            ToastOptions::default().duration(Duration::from_millis(3000)),
                        );
                        is_loading.set(false);
                    }
                }
            });
        }
    };

    use_effect(move || {
        if submit_attempted() {
            validate_email();
            validate_password();
        }
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "Change Email" }
                }

                div { class: "screen-content centered content-enter",
                div { class : "container-sm",

                    form { class: "flex-col text-center",

                        if submit_attempted() {
                            if let Some(error) = email_error() {
                                div { class : "message-error", "{error}" }
                            }
                        }

                        TextInput {
                            value: new_email,
                            id: "new_email",
                            label: "New email",
                            placeholder: "New email",
                        }

                        if submit_attempted() {
                            if let Some(error) = password_error() {
                                div { class : "message-error", "{error}" }
                            }
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

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
                button {
                    class: "util-btn",
                    disabled: is_loading(),
                    onclick: move |_| attempt_submit(),
                    if is_loading() { "Saving..." } else { "Save changes" }
                }
            }
            }
        }
    }
}
