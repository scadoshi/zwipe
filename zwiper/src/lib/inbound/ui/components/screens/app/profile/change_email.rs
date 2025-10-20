use crate::inbound::ui::router::Router;
use dioxus::prelude::*;
use email_address::EmailAddress;
use std::str::FromStr;
use zwipe::domain::auth::models::{password::Password, session::Session};

#[component]
pub fn ChangeEmail() -> Element {
    let navigator = use_navigator();
    let _session: Signal<Option<Session>> = use_context();

    let mut new_email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_attempted = use_signal(|| false);

    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let mut validate_email = move || {
        if let Err(e) = EmailAddress::from_str(&new_email.read()) {
            email_error.set(Some(format!("invalid email: {}", e)));
        } else {
            email_error.set(None)
        }
    };

    let inputs_are_valid = move || {
        EmailAddress::from_str(&new_email.read()).is_ok() && Password::new(&password.read()).is_ok()
    };

    rsx! {
        div { class : "nicely-centered",
            div { class : "form-container",
                h2 { "change email" }

                if *submit_attempted.read() {
                    if let Some(error) = email_error.read().as_ref() {
                        div { class : "error", "{error}" }
                    }
                }

                form {
                    div { class : "form-group",
                        input {
                            id : "new_email",
                            r#type : "text",
                            placeholder : "new email",
                            value : "{new_email}",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput: move |event| {
                                new_email.set(event.value());
                                if *submit_attempted.read() {
                                    validate_email();
                                }
                            }
                        }

                        input {
                            id : "password",
                            r#type : "password",
                            placeholder : "password",
                            value : "{password}",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                password.set(event.value());
                            }
                        }

                        button {
                            onclick : move |_| {
                                submit_attempted.set(true);
                                validate_email();
                                if inputs_are_valid() {
                                    // TODO: Implement API call
                                    tracing::info!("change email to: {}", new_email.read());
                                    submission_error.set(Some("not implemented yet".to_string()));
                                } else {
                                    submission_error.set(Some("invalid input".to_string()));
                                }
                            }, "submit"
                        }

                        button {
                            onclick : move |_| {
                                navigator.push(Router::Profile {});
                            }, "back"
                        }
                    }
                }

                if let Some(error) = submission_error.read().as_deref() {
                    div { class: "error", "{error}" }
                }
            }
        }
    }
}
