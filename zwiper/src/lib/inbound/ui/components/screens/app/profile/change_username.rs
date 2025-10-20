use crate::inbound::ui::router::Router;
use dioxus::prelude::*;
use zwipe::domain::{
    auth::models::{password::Password, session::Session},
    user::models::username::Username,
};

#[component]
pub fn ChangeUsername() -> Element {
    let navigator = use_navigator();
    let _session: Signal<Option<Session>> = use_context();

    let mut new_username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_attempted = use_signal(|| false);

    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let mut validate_username = move || {
        if let Err(e) = Username::new(&new_username.read()) {
            username_error.set(Some(e.to_string()));
        } else {
            username_error.set(None)
        }
    };

    let inputs_are_valid = move || {
        Username::new(&new_username.read()).is_ok() && Password::new(&password.read()).is_ok()
    };

    rsx! {
        div { class : "nicely-centered",
            div { class : "form-container",
                h2 { "change username" }

                if *submit_attempted.read() {
                    if let Some(error) = username_error.read().as_ref() {
                        div { class : "error", "{error}" }
                    }
                }

                form {
                    div { class : "form-group",
                        input {
                            id : "new_username",
                            r#type : "text",
                            placeholder : "new username",
                            value : "{new_username}",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput: move |event| {
                                new_username.set(event.value());
                                if *submit_attempted.read() {
                                    validate_username();
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
                                validate_username();
                                if inputs_are_valid() {
                                    // TODO: Implement API call
                                    tracing::info!("change username to: {}", new_username.read());
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
