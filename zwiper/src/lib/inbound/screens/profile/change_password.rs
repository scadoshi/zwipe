//! Change password screen.

use crate::{
    inbound::components::{
        alert_dialog::{
            AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
            AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
        },
        auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
        fields::text_input::TextInput,
    },
    outbound::client::{ZwipeClient, user::change_password::ClientChangePassword},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe::domain::auth::models::password::Password;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpChangePassword;

/// Form screen for updating user's password.
#[component]
pub fn ChangePassword() -> Element {
    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    // we do not validate current password on frontend
    // as to not lock them out of changing
    // if their current somehow defies policy
    let mut current_password = use_signal(String::new);

    let mut new_password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_new_password = move || {
        if let Err(e) = Password::new(new_password()) {
            password_error.set(Some(e.to_string()));
        } else if new_password().as_str() != confirm_password().as_str() {
            password_error.set(Some("Passwords do not match".to_string()));
        } else if new_password().as_str() == current_password().as_str() {
            password_error.set(Some(
                "New password must be different from your current password".to_string(),
            ));
        } else {
            password_error.set(None)
        }
    };

    let mut submit_attempted = use_signal(|| false);
    let mut show_confirm = use_signal(|| false);
    let mut is_loading = use_signal(|| false);
    let toast = use_toast();

    let mut inputs_are_valid = move || {
        validate_new_password();
        password_error().is_none()
    };

    let mut clear_inputs = move || {
        current_password.set(String::new());
        new_password.set(String::new());
        confirm_password.set(String::new());
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        if inputs_are_valid() {
            tracing::info!("change password");
            let request = HttpChangePassword::new(&current_password(), &new_password());
            is_loading.set(true);
            spawn(async move {
                let session = match session.ensure_fresh(auth_client).await {
                    Ok(session) => session,
                    Err(e) => {
                        toast.error(
                            e.to_user_message(),
                            ToastOptions::default().duration(Duration::from_millis(3000)),
                        );
                        is_loading.set(false);
                        return;
                    }
                };

                match auth_client().change_password(request, &session).await {
                    Ok(()) => {
                        toast.success(
                            "Password change successful".to_string(),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                        clear_inputs();
                        submit_attempted.set(false);
                        is_loading.set(false);
                    }
                    Err(e) => {
                        tracing::warn!("change password failed: {e}");
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
            validate_new_password();
        }
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "Change Password" }
                }

                div { class: "screen-content centered content-enter",
                div { class : "container-sm",

                    form { class: "flex-col text-center",

                        TextInput {
                            value: current_password,
                            id: "current_password",
                            label: "Current password",
                            placeholder: "Current password",
                            input_type: "password",
                        }

                        if submit_attempted() {
                            if let Some(error) = password_error() {
                                div { class : "message-error", "{error}" }
                            }
                        }

                        TextInput {
                            value: new_password,
                            id: "new_password",
                            label: "New password",
                            placeholder: "New password",
                            input_type: "password",
                        }

                        TextInput {
                            value: confirm_password,
                            id: "confirm_password",
                            label: "Confirm password",
                            placeholder: "Confirm new",
                            input_type: "password",
                        }
                    }

                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    disabled: is_loading(),
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
                button {
                    class: "util-btn",
                    disabled: is_loading(),
                    onclick: move |_| {
                        submit_attempted.set(true);
                        validate_new_password();
                        if password_error().is_none() {
                            show_confirm.set(true);
                        }
                    },
                    if is_loading() { "Saving..." } else { "Save changes" }
                }
            }

            AlertDialogRoot {
                open: show_confirm(),
                on_open_change: move |open| show_confirm.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "Change password" }
                    AlertDialogDescription {
                        "Changing your password will log you out on all other devices."
                    }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_confirm.set(false),
                            "Cancel"
                        }
                        AlertDialogAction {
                            on_click: move |_| {
                                show_confirm.set(false);
                                attempt_submit();
                            },
                            "Confirm"
                        }
                    }
                }
            }
            }
        }
    }
}
