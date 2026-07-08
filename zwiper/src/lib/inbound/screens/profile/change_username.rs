//! Change username bottom sheet.

use crate::{
    inbound::components::{
        auth::ensure_session::EnsureFresh, bottom_sheet::BottomSheet, fields::text_input::TextInput,
    },
    outbound::client::{ZwipeClient, user::change_username::ClientChangeUsername},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{Button, ButtonVariant};
use zwipe_core::{
    domain::{auth::models::session::Session, user::username::Username},
    http::contracts::auth::HttpChangeUsername,
};

/// Bottom sheet for updating the user's username.
#[component]
pub fn ChangeUsernameSheet(mut open: Signal<bool>) -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let mut new_username = use_signal(String::new);
    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut username_touched = use_signal(|| false);
    let mut validate_username = move || {
        if let Err(e) = Username::new(new_username()) {
            username_error.set(Some(e.to_string()));
        } else {
            username_error.set(None)
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
        validate_username();
        validate_password();
        username_error().is_none() && password_error().is_none()
    };

    let mut clear_inputs = move || {
        new_username.set(String::new());
        password.set(String::new());
    };

    // Reset the form each time the sheet opens so it never shows stale input.
    use_effect(move || {
        if open() {
            clear_inputs();
            username_error.set(None);
            password_error.set(None);
            username_touched.set(false);
            password_touched.set(false);
            submit_attempted.set(false);
        }
    });

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        if inputs_are_valid() {
            tracing::info!("change username to {}", new_username());
            let request = HttpChangeUsername::new(&new_username(), &password());
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

                match auth_client().change_username(request, &session_value).await {
                    Ok(updated_user) => {
                        let new_name = updated_user.username.clone();
                        session_value.user.username = updated_user.username;
                        session.set(Some(session_value));
                        toast.success(
                            format!("Username changed to {}", new_name),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                        clear_inputs();
                        submit_attempted.set(false);
                        is_loading.set(false);
                        open.set(false);
                    }
                    Err(e) => {
                        tracing::warn!("change username failed: {e}");
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
        let value = new_username();
        if !value.is_empty() && !username_touched() {
            username_touched.set(true);
        }
        if username_touched() || submit_attempted() {
            validate_username();
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
            title: "Change username".to_string(),
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
                    value: new_username,
                    id: "new_username",
                    label: "New username",
                    placeholder: "New username",
                    error: username_error(),
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
