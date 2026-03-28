//! Change username screen.

use crate::{
    inbound::components::{
        auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        fields::text_input::TextInput,
    },
    outbound::client::{user::change_username::ClientChangeUsername, ZwipeClient},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe::{
    domain::{
        auth::models::{password::Password, session::Session},
        user::models::username::Username,
    },
    inbound::http::handlers::auth::change_username::HttpChangeUsername,
};

/// Form screen for updating user's username.
#[component]
pub fn ChangeUsername() -> Element {
    let navigator = use_navigator();

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
            password_error.set(Some("invalid password".to_string()));
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

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        if inputs_are_valid() {
            tracing::info!("change username to {}", new_username());
            let request = HttpChangeUsername::new(&new_username(), &password());
            is_loading.set(true);
            spawn(async move {
                session.upkeep(auth_client);
                let Some(mut session_value) = session() else {
                    toast.error(
                        "session expired — please log in again".to_string(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    is_loading.set(false);
                    return;
                };

                match auth_client().change_username(request, &session_value).await {
                    Ok(updated_user) => {
                        let new_name = updated_user.username.clone();
                        session_value.user.username = updated_user.username;
                        session.set(Some(session_value));
                        toast.success(
                            format!("username changed to {}", new_name),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                        clear_inputs();
                        submit_attempted.set(false);
                        is_loading.set(false);
                    }
                    Err(e) => {
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
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "change username" }
                }

                div { class: "screen-content centered",
                div { class : "container-sm",

                    form { class: "flex-col text-center",

                        if submit_attempted() {
                            if let Some(error) = username_error() {
                                div { class : "message-error", "{error}" }
                            }
                        }

                        TextInput {
                            value: new_username,
                            id: "new_username",
                            label: "new username",
                            placeholder: "new username",
                        }

                        if submit_attempted() {
                            if let Some(error) = password_error() {
                                div { class : "message-error", "{error}" }
                            }
                        }

                        TextInput {
                            value: password,
                            id: "password",
                            label: "password",
                            placeholder: "password",
                            input_type: "password",
                        }
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
                    onclick: move |_| attempt_submit(),
                    if is_loading() { "saving..." } else { "save changes" }
                }
            }
            }
        }
    }
}
