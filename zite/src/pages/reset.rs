use crate::{Nav, api};
use dioxus::prelude::*;
use zwipe_client::ClientError;
use zwipe_components::{InfoButton, Panel};
use zwipe_core::domain::auth::{models::secret::Secret, password::requirements};

/// Validate a candidate password against the shared password policy.
fn validate_password(pw: &str) -> Option<String> {
    zwipe_core::domain::auth::password::validate(pw)
        .err()
        .map(|e| format!("Password {e}"))
}

#[derive(Clone, PartialEq)]
enum ResetState {
    Form,
    Loading,
    Success,
    Error(String),
}

#[component]
pub fn Reset(token: String) -> Element {
    let mut password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut state = use_signal(|| ResetState::Form);
    let mut rules_open = use_signal(|| false);

    let on_submit = move |e: FormEvent| {
        e.prevent_default();

        let pw = password.read().clone();
        let cf = confirm.read().clone();

        if let Some(err) = validate_password(&pw) {
            state.set(ResetState::Error(err));
            return;
        }
        if pw != cf {
            state.set(ResetState::Error("Passwords do not match".to_string()));
            return;
        }

        let token = token.clone();
        if token.is_empty() {
            state.set(ResetState::Error(
                "This link is missing its code. Open it from the email again.".to_string(),
            ));
            return;
        }

        state.set(ResetState::Loading);

        spawn(async move {
            match api::client().reset_password(token, Secret::new(pw)).await {
                Ok(()) => state.set(ResetState::Success),
                // A request that never landed is not a bad token; say so.
                Err(e @ (ClientError::Network(_) | ClientError::Decode(_))) => {
                    state.set(ResetState::Error(e.to_user_message()))
                }
                Err(_) => state.set(ResetState::Error(
                    "This link is invalid or has expired. Request a new one from the app."
                        .to_string(),
                )),
            }
        });
    };

    let current_state = state.read().clone();

    rsx! {
        Nav {}
        // The password policy, from the same list the app's hint shows. A tap
        // on the backdrop or Got it closes it.
        if rules_open() {
            div { class: "sd-image-overlay-backdrop" }
            div { class: "hint-overlay", onclick: move |_| rules_open.set(false),
                div { class: "hint-dialog", onclick: move |e| e.stop_propagation(),
                    Panel {
                        title: "Password rules",
                        centered: true,
                        actions: rsx! {
                            button {
                                class: "panel-action",
                                r#type: "button",
                                onclick: move |_| rules_open.set(false),
                                "Got it"
                            }
                        },
                        ul { class: "card-bullets",
                            for line in requirements() {
                                li { "{line}" }
                            }
                        }
                    }
                }
            }
        }
        div { class: "form-page content-enter",
            match current_state {
                ResetState::Success => rsx! {
                    Panel { title: "Password reset", title_h1: true, centered: true,
                        p { class: "card-summary", "Your password is updated and every session is signed out." }
                    }
                },
                _ => rsx! {
                    Panel { title: "Reset password", title_h1: true, centered: true,
                        actions: rsx! {
                            button {
                                r#type: "submit",
                                form: "reset-form",
                                class: "panel-action",
                                disabled: current_state == ResetState::Loading,
                                if current_state == ResetState::Loading { "Updating..." } else { "Set new password" }
                            }
                        },

                    form { id: "reset-form", class: "form-centered", onsubmit: on_submit,
                        div { class: "form-group",
                            div { class: "label-row",
                                span { class: "label-hint-anchor",
                                    label { "New password" }
                                    InfoButton { onclick: move |_| rules_open.set(true) }
                                }
                            }
                            input {
                                r#type: "password",
                                placeholder: "New password",
                                value: "{password}",
                                oninput: move |e| password.set(e.value()),
                                disabled: current_state == ResetState::Loading,
                            }
                        }
                        div { class: "form-group",
                            label { "Confirm password" }
                            input {
                                r#type: "password",
                                placeholder: "Confirm password",
                                value: "{confirm}",
                                oninput: move |e| confirm.set(e.value()),
                                disabled: current_state == ResetState::Loading,
                            }
                        }
                    }

                    if let ResetState::Error(msg) = &*state.read() {
                        div { class: "status-message error", "{msg}" }
                    }
                    }
                },
            }
        }
    }
}
