use dioxus::prelude::*;
use serde::Serialize;
use crate::{API_BASE, Nav};

#[derive(Serialize)]
struct ResetPasswordRequest {
    token: String,
    new_password: String,
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

    let on_submit = move |e: FormEvent| {
        e.prevent_default();

        let pw = password.read().clone();
        let cf = confirm.read().clone();

        if pw != cf {
            state.set(ResetState::Error("passwords do not match".to_string()));
            return;
        }
        if pw.len() < 8 {
            state.set(ResetState::Error("password must be at least 8 characters".to_string()));
            return;
        }

        let token = token.clone();
        if token.is_empty() {
            state.set(ResetState::Error("no token found in url".to_string()));
            return;
        }

        state.set(ResetState::Loading);

        spawn(async move {
            let client = reqwest::Client::new();
            let res = client
                .post(format!("{API_BASE}/api/auth/reset-password"))
                .json(&ResetPasswordRequest { token, new_password: pw })
                .send()
                .await;

            match res {
                Ok(r) if r.status().is_success() => state.set(ResetState::Success),
                Ok(_) => state.set(ResetState::Error("token not found or expired".to_string())),
                Err(e) => state.set(ResetState::Error(e.to_string())),
            }
        });
    };

    let current_state = state.read().clone();

    rsx! {
        Nav {}
        div { class: "form-page",
            match current_state {
                ResetState::Success => rsx! {
                    h1 { "password reset" }
                    p { class: "subtitle", "your password has been updated and all sessions have been signed out." }
                    div { class: "status-message success", "password updated successfully" }
                },
                _ => rsx! {
                    h1 { "reset password" }
                    p { class: "subtitle", "enter your new password below." }

                    form { onsubmit: on_submit,
                        div { class: "form-group",
                            label { "new password" }
                            input {
                                r#type: "password",
                                placeholder: "at least 8 characters",
                                value: "{password}",
                                oninput: move |e| password.set(e.value()),
                                disabled: current_state == ResetState::Loading,
                            }
                        }
                        div { class: "form-group",
                            label { "confirm password" }
                            input {
                                r#type: "password",
                                placeholder: "repeat password",
                                value: "{confirm}",
                                oninput: move |e| confirm.set(e.value()),
                                disabled: current_state == ResetState::Loading,
                            }
                        }
                        button {
                            r#type: "submit",
                            class: "btn-primary",
                            disabled: current_state == ResetState::Loading,
                            if current_state == ResetState::Loading { "updating..." } else { "set new password" }
                        }
                    }

                    if let ResetState::Error(msg) = &*state.read() {
                        div { class: "status-message error", "{msg}" }
                    }
                },
            }
        }
    }
}
