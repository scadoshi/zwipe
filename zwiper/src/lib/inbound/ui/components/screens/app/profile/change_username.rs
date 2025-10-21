use crate::{
    inbound::ui::{
        components::{
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
            success_messages::get_random_success_message,
        },
        router::Router,
    },
    outbound::client::auth::{
        change_username::{ChangeUsername as ChangeUsernameTrait, ChangeUsernameError},
        session::ActiveSession,
        AuthClient,
    },
};
use dioxus::prelude::*;
use zwipe::{
    domain::{
        auth::models::{password::Password, session::Session},
        user::models::username::Username,
    },
    inbound::http::handlers::auth::change_username::HttpChangeUsername,
};

#[component]
pub fn ChangeUsername() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<AuthClient> = use_context();

    let mut new_username = use_signal(|| String::new());
    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_username = move || {
        if let Err(e) = Username::new(&new_username.read()) {
            username_error.set(Some(e.to_string()));
        } else {
            username_error.set(None)
        }
    };

    let mut password = use_signal(|| String::new());
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_password = move || {
        if let Err(_) = Password::new(&password.read()) {
            password_error.set(Some("invalid password".to_string()));
        } else {
            password_error.set(None);
        }
    };

    let mut submission_error: Signal<Option<String>> = use_signal(|| None);
    let mut success_message: Signal<Option<String>> = use_signal(|| None);
    let mut submit_attempted = use_signal(|| false);

    let mut inputs_are_valid = move || {
        validate_username();
        validate_password();
        username_error.read().is_none() && password_error.read().is_none()
    };

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "form-container",

                h2 { "change username" }

                form {
                    div { class : "form-group",

                        if *submit_attempted.read() {
                            if let Some(error) = username_error.read().as_ref() {
                                div { class : "error", "{error}" }
                            }
                        }

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

                        if *submit_attempted.read() {
                            if let Some(error) = password_error.read().as_ref() {
                                div { class : "error", "{error}" }
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
                                if *submit_attempted.read() {
                                    validate_password();
                                }
                            }
                        }

                        button {
                            onclick : move |_| {
                                submit_attempted.set(true);
                                if inputs_are_valid() {
                                    tracing::info!("change username to {}", new_username.read());
                                    let request = HttpChangeUsername::new(&*new_username.read(), &*password.read());
                                    spawn(async move {
                                        let Some(current) = session.read().clone() else {
                                            submission_error.set(Some(ChangeUsernameError::SessionExpired.to_string()));
                                            return;
                                        };

                                        let Some(mut active) = auth_client
                                            .read()
                                            .infallible_get_active_session(&current)
                                            .await else { return; };

                                        match auth_client.read().change_username(request, &active).await {
                                            Ok(updated_user) => {
                                                active.user.username = updated_user.username;
                                                success_message.set(Some(get_random_success_message()));
                                                submission_error.set(None);
                                            }
                                            Err(e) => submission_error.set(Some(e.to_string())),
                                        }

                                        session.set(Some(active));
                                    });

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
                } else if let Some(success_message) = success_message.read().as_deref() {
                    div { class: "success-message", {success_message} }
                }
            }
        }
    }
}
