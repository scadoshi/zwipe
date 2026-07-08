//! Change email bottom sheet.

use crate::{
    domain::error::UserFacing,
    inbound::components::{
        auth::ensure_session::EnsureFresh, bottom_sheet::BottomSheet, fields::text_input::TextInput,
    },
    outbound::client::{ZwipeClient, user::change_email::ClientChangeEmail},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{Button, ButtonVariant};
use zwipe_core::{
    domain::{Email, auth::models::session::Session},
    http::contracts::auth::HttpChangeEmail,
};

/// Bottom sheet for updating the user's email address.
#[component]
pub fn ChangeEmailSheet(mut open: Signal<bool>) -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let mut new_email = use_signal(String::new);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut email_touched = use_signal(|| false);
    let mut validate_email = move || {
        if let Err(e) = Email::new(new_email()) {
            email_error.set(Some(e.to_user_facing_string()));
        } else {
            email_error.set(None)
        }
    };

    let mut password = use_signal(String::new);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut password_touched = use_signal(|| false);
    let mut validate_password = move || {
        if password().is_empty() {
            password_error.set(Some("Password is required".to_string()));
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
            email_touched.set(false);
            password_touched.set(false);
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
        let value = new_email();
        if !value.is_empty() && !email_touched() {
            email_touched.set(true);
        }
        if email_touched() || submit_attempted() {
            validate_email();
        }
    });

    use_effect(move || {
        let value = password();
        if !value.is_empty() && !password_touched() {
            password_touched.set(true);
        }
        if password_touched() || submit_attempted() {
            validate_password();
        }
    });

    rsx! {
        BottomSheet {
            open,
            title: "Change email".to_string(),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Util,
                    disabled: is_loading(),
                    onclick: move |_| open.set(false),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    disabled: is_loading(),
                    onclick: move |_| attempt_submit(),
                    if is_loading() { "Saving..." } else { "Save changes" }
                }
            },

            div { class: "flex-col text-center",

                TextInput {
                    value: new_email,
                    id: "new_email",
                    label: "New email",
                    placeholder: "New email",
                    error: email_error(),
                }

                TextInput {
                    value: password,
                    id: "password",
                    label: "Password",
                    placeholder: "Password",
                    input_type: "password",
                    error: password_error(),
                }
            }
        }
    }
}
