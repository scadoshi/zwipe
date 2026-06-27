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
use zwipe::domain::auth::models::password::Password;
use zwipe_core::domain::{auth::models::session::Session, user::username::Username};
use zwipe_core::http::contracts::auth::HttpChangeUsername;

/// Bottom sheet for updating the user's username.
#[component]
pub fn ChangeUsernameSheet(mut open: Signal<bool>) -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let mut new_username = use_signal(String::new);
    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_username = move || {
        if let Err(e) = Username::new(new_username()) {
            username_error.set(Some(e.to_string()));
        } else {
            username_error.set(None)
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
        if submit_attempted() {
            validate_username();
            validate_password();
        }
    });

    rsx! {
        BottomSheet {
            open,
            title: "Change username".to_string(),
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
                    if let Some(error) = username_error() {
                        div { class: "message-error", "{error}" }
                    }
                }

                TextInput {
                    value: new_username,
                    id: "new_username",
                    label: "New username",
                    placeholder: "New username",
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
