//! Change password screen.

use crate::{
    inbound::components::{
        alert_dialog::{
            AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
            AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
        },
        auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        fields::text_input::TextInput,
    },
    outbound::client::{user::change_password::ClientChangePassword, ZwipeClient},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe::{
    domain::auth::models::{password::Password, session::Session},
    inbound::http::{handlers::auth::change_password::HttpChangePassword, ApiError},
};

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
            password_error.set(Some("passwords do not match".to_string()));
        } else {
            password_error.set(None)
        }
    };

    let mut submission_error: Signal<Option<String>> = use_signal(|| None);
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
                session.upkeep(auth_client);
                let Some(session) = session() else {
                    submission_error.set(Some(
                        ApiError::Unauthorized("session expired".to_string()).to_string(),
                    ));
                    is_loading.set(false);
                    return;
                };

                match auth_client().change_password(request, &session).await {
                    Ok(()) => {
                        toast.success(
                            "password change successful".to_string(),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                        submission_error.set(None);
                        clear_inputs();
                        submit_attempted.set(false);
                        is_loading.set(false);
                    }
                    Err(e) => {
                        submission_error.set(Some(e.to_string()));
                        is_loading.set(false);
                    }
                }
            });
        } else {
            submission_error.set(Some("invalid input".to_string()));
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
                    h2 { "change password" }
                }

                div { class: "screen-content centered",
                div { class : "container-sm",

                    form { class: "flex-col text-center",

                        TextInput {
                            value: current_password,
                            id: "current_password",
                            label: "current password",
                            placeholder: "current password",
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
                            label: "new password",
                            placeholder: "new password",
                            input_type: "password",
                        }

                        TextInput {
                            value: confirm_password,
                            id: "confirm_password",
                            label: "confirm password",
                            placeholder: "confirm new",
                            input_type: "password",
                        }
                    }

                    if let Some(error) = submission_error() {
                        div { class: "message-error", "{error}" }
                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "back"
                }
                button {
                    class: "util-btn",
                    disabled: is_loading(),
                    onclick: move |_| {
                        submit_attempted.set(true);
                        validate_new_password();
                        if password_error().is_none() {
                            show_confirm.set(true);
                        } else {
                            submission_error.set(Some("invalid input".to_string()));
                        }
                    },
                    if is_loading() { "saving..." } else { "save changes" }
                }
            }

            AlertDialogRoot {
                open: show_confirm(),
                on_open_change: move |open| show_confirm.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "change password" }
                    AlertDialogDescription {
                        "changing your password will log you out on all other devices."
                    }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_confirm.set(false),
                            "cancel"
                        }
                        AlertDialogAction {
                            on_click: move |_| {
                                show_confirm.set(false);
                                attempt_submit();
                            },
                            "confirm"
                        }
                    }
                }
            }
            }
        }
    }
}
