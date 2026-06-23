//! Change email bottom sheet.

use crate::domain::error::UserFacing;
use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::bottom_sheet::BottomSheet;
use crate::inbound::components::fields::text_input::TextInput;
use crate::outbound::client::user::change_email::ClientChangeEmail;
use crate::outbound::client::ZwipeClient;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe::domain::auth::models::password::Password;
use zwipe_core::http::contracts::auth::HttpChangeEmail;
use zwipe_core::domain::{auth::models::session::Session, Email};

/// Bottom sheet for updating the user's email address.
#[component]
pub fn ChangeEmailSheet(mut open: Signal<bool>) -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let mut new_email = use_signal(String::new);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_email = move || {
        if let Err(e) = Email::new(new_email()) {
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

    // Reset the form each time the sheet opens so it never shows stale input.
    use_effect(move || {
        if open() {
            clear_inputs();
            email_error.set(None);
            password_error.set(None);
            submit_attempted.set(false);
        }
    });

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        if inputs_are_valid() {
            tracing::info!("change email to {}", new_email());
            let request = HttpChangeEmail::new(&new_email(), &password());
            is_loading.set(true);
            spawn(async move {
                let mut session_value = match session.ensure_fresh(auth_client).await {
                    Ok(session_value) => session_value,
                    Err(e) => {
                        toast.error(
                            e.to_user_message(),
                            ToastOptions::default().duration(Duration::from_millis(3000)),
                        );
                        is_loading.set(false);
                        return;
                    }
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
                        open.set(false);
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
        BottomSheet {
            open,
            title: "Change email".to_string(),
            footer: rsx! {
                button {
                    class: "util-btn",
                    disabled: is_loading(),
                    onclick: move |_| open.set(false),
                    "Back"
                }
                button {
                    class: "util-btn",
                    disabled: is_loading(),
                    onclick: move |_| attempt_submit(),
                    if is_loading() { "Saving..." } else { "Save changes" }
                }
            },

            div { class: "flex-col text-center",

                if submit_attempted() {
                    if let Some(error) = email_error() {
                        div { class: "message-error", "{error}" }
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
                        div { class: "message-error", "{error}" }
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
}
