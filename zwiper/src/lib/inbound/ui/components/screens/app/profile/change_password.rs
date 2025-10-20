use crate::inbound::ui::router::Router;
use dioxus::prelude::*;
use zwipe::domain::auth::models::{password::Password, session::Session};

#[component]
pub fn ChangePassword() -> Element {
    let navigator = use_navigator();
    let _session: Signal<Option<Session>> = use_context();

    let mut current_password = use_signal(|| String::new());
    let mut new_password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());

    let mut submit_attempted = use_signal(|| false);

    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let mut validate_new_password = move || {
        if let Err(e) = Password::new(&new_password.read()) {
            password_error.set(Some(e.to_string()));
        } else if new_password.read().as_str() != confirm_password.read().as_str() {
            password_error.set(Some("passwords do not match".to_string()));
        } else {
            password_error.set(None)
        }
    };

    let inputs_are_valid = move || {
        Password::new(&current_password.read()).is_ok()
            && Password::new(&new_password.read()).is_ok()
            && new_password.read().as_str() == confirm_password.read().as_str()
    };

    rsx! {
        div { class : "nicely-centered",
            div { class : "form-container",
                h2 { "change password" }

                if *submit_attempted.read() {
                    if let Some(error) = password_error.read().as_ref() {
                        div { class : "error", "{error}" }
                    }
                }

                form {
                    div { class : "form-group",
                        input {
                            id : "current_password",
                            r#type : "password",
                            placeholder : "current",
                            value : "{current_password}",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput: move |event| {
                                current_password.set(event.value());
                            }
                        }

                        input {
                            id : "new_password",
                            r#type : "password",
                            placeholder : "new",
                            value : "{new_password}",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                new_password.set(event.value());
                                if *submit_attempted.read() {
                                    validate_new_password();
                                }
                            }
                        }

                        input {
                            id : "confirm_password",
                            r#type : "password",
                            placeholder : "confirm new",
                            value : "{confirm_password}",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                confirm_password.set(event.value());
                                if *submit_attempted.read() {
                                    validate_new_password();
                                }
                            }
                        }

                        button {
                            onclick : move |_| {
                                submit_attempted.set(true);
                                validate_new_password();
                                if inputs_are_valid() {
                                    // TODO: Implement API call
                                    tracing::info!("change password");
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
