use components::{InfoButton, Panel, PasswordInput};
use dioxus::prelude::*;
use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::secret::Secret;

use crate::{Nav, api};

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

/// What a submit does.
#[derive(Debug, PartialEq)]
enum SubmitCheck {
    /// A reset is already out: ignore the submit, the token is single use.
    Ignore,
    /// Show this error instead of sending.
    Reject(String),
    /// Send the reset.
    Send,
}

/// Decides a submit from the page state, the two fields and the link's token.
fn check_submit(state: &ResetState, pw: &str, cf: &str, token: &str) -> SubmitCheck {
    if *state == ResetState::Loading {
        return SubmitCheck::Ignore;
    }
    if let Some(err) = validate_password(pw) {
        return SubmitCheck::Reject(err);
    }
    if pw != cf {
        return SubmitCheck::Reject("Passwords do not match".to_string());
    }
    if token.is_empty() {
        return SubmitCheck::Reject(
            "This link is missing its code. Open it from the email again.".to_string(),
        );
    }
    SubmitCheck::Send
}

#[component]
pub fn Reset(token: String) -> Element {
    let password = use_signal(String::new);
    let confirm = use_signal(String::new);
    let mut state = use_signal(|| ResetState::Form);
    let mut rules_open = use_signal(|| false);

    let on_submit = move |e: FormEvent| {
        e.prevent_default();

        let pw = password.read().clone();
        let cf = confirm.read().clone();
        let token = token.clone();

        let check = check_submit(&state.peek(), &pw, &cf, &token);
        match check {
            SubmitCheck::Ignore => return,
            SubmitCheck::Reject(err) => {
                state.set(ResetState::Error(err));
                return;
            }
            SubmitCheck::Send => {}
        }

        state.set(ResetState::Loading);

        spawn(async move {
            match api::client().reset_password(token, Secret::new(pw)).await {
                Ok(()) => state.set(ResetState::Success),
                // A request that never landed is not a bad token; say so.
                Err(e @ (ClientError::Network(_) | ClientError::Decode(_))) => {
                    state.set(ResetState::Error(e.to_user_message()));
                }
                Err(_) => state.set(ResetState::Error(
                    "This link is invalid or has expired. Request a new one from the app."
                        .to_string(),
                )),
            }
        });
    };

    // Editing either field clears a submit error, so the live check below
    // isn't hidden behind a stale one.
    use_effect(move || {
        let _ = (password.read().len(), confirm.read().len());
        if matches!(*state.peek(), ResetState::Error(_)) {
            state.set(ResetState::Form);
        }
    });

    let current_state = state.read().clone();

    // Same rule as the app: mid-typing the confirmation can't match yet, so
    // it is judged once it is as long as the password (submit checks too).
    let mismatch = {
        let pw = password.read();
        let cf = confirm.read();
        !cf.is_empty() && cf.chars().count() >= pw.chars().count() && *pw != *cf
    };

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
                            for line in zwipe_core::domain::auth::password::requirements() {
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
                    div { class: "outcome-success",
                    Panel { title: "Password reset", title_h1: true, centered: true,
                        p { class: "card-summary", "Your password is updated and every session is signed out." }
                    }
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
                                    label { r#for: "reset-password", "New password" }
                                    InfoButton { onclick: move |_| rules_open.set(true) }
                                }
                            }
                            PasswordInput {
                                value: password,
                                id: "reset-password",
                                placeholder: "New password",
                                disabled: current_state == ResetState::Loading,
                            }
                        }
                        div { class: "form-group",
                            label { r#for: "reset-confirm", "Confirm password" }
                            PasswordInput {
                                value: confirm,
                                id: "reset-confirm",
                                placeholder: "Confirm password",
                                disabled: current_state == ResetState::Loading,
                            }
                        }
                    }

                    if let ResetState::Error(msg) = &*state.read() {
                        div { class: "status-message error", "{msg}" }
                    } else if mismatch {
                        div { class: "status-message error", "Passwords do not match" }
                    }
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "Correct!Horse!Battery9";

    #[test]
    fn a_submit_while_loading_is_ignored() {
        assert_eq!(
            check_submit(&ResetState::Loading, GOOD, GOOD, "token"),
            SubmitCheck::Ignore,
            "a second send would burn the single-use token"
        );
    }

    #[test]
    fn an_idle_submit_checks_the_form() {
        assert_eq!(
            check_submit(&ResetState::Form, GOOD, GOOD, "token"),
            SubmitCheck::Send
        );
        assert_eq!(
            check_submit(&ResetState::Form, GOOD, "other", "token"),
            SubmitCheck::Reject("Passwords do not match".to_string())
        );
        assert!(matches!(
            check_submit(&ResetState::Form, GOOD, GOOD, ""),
            SubmitCheck::Reject(_)
        ));
    }
}
