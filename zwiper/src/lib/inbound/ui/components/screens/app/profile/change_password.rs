use crate::{
    inbound::ui::{
        components::{
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
            success_messages::get_random_success_message,
        },
        router::Router,
    },
    outbound::client::auth::{
        change_password::{ChangePassword as ChangePasswordTrait, ChangePasswordError},
        session::ActiveSession,
        AuthClient,
    },
};
use dioxus::prelude::*;
use zwipe::{
    domain::auth::models::{password::Password, session::Session},
    inbound::http::handlers::auth::change_password::HttpChangePassword,
};

#[component]
pub fn ChangePassword() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<AuthClient> = use_context();

    // we do not validate current in case their current
    // somehow doesn't follow password policies
    // we don't want to lock them to it
    let mut current_password = use_signal(|| String::new());

    let mut new_password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_new_password = move || {
        if let Err(e) = Password::new(&new_password.read()) {
            password_error.set(Some(e.to_string()));
        } else if new_password.read().as_str() != confirm_password.read().as_str() {
            password_error.set(Some("passwords do not match".to_string()));
        } else {
            password_error.set(None)
        }
    };

    let mut submission_error: Signal<Option<String>> = use_signal(|| None);
    let mut success_message: Signal<Option<String>> = use_signal(|| None);
    let mut submit_attempted = use_signal(|| false);

    let mut inputs_are_valid = move || {
        validate_new_password();
        Password::new(&current_password.read()).is_ok() && password_error.read().is_none()
    };

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "form-container",

                h2 { "change password" }

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

                        if *submit_attempted.read() {
                            if let Some(error) = password_error.read().as_ref() {
                                div { class : "error", "{error}" }
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
                                if inputs_are_valid() {
                                    tracing::info!("change password");
                                    let request = HttpChangePassword::new(
                                        &*current_password.read(),
                                        &*new_password.read()
                                    );
                                    spawn(async move {
                                        let Some(current) = session.read().clone() else {
                                            submission_error.set(Some(ChangePasswordError::SessionExpired.to_string()));
                                            return;
                                        };

                                        let Some(active) = auth_client
                                            .read()
                                            .infallible_get_active_session(&current)
                                            .await else { return; };

                                        match auth_client.read().change_password(request, &active).await {
                                            Ok(()) => {
                                                success_message.set(Some(get_random_success_message()));
                                                submission_error.set(None);
                                            }
                                            Err(e) => submission_error.set(Some(e.to_string())),
                                        }

                                        if active != current {
                                            session.set(Some(active));
                                        }
                                    });

                                } else {
                                    submission_error.set(Some("invalid input".to_string()));
                                }
                            }, "submit"
                        }

                        if let Some(error) = submission_error.read().as_deref() {
                            div { class: "error", "{error}" }
                        } else if let Some(success_message) = success_message.read().as_deref() {
                            div { class: "success-message", {success_message} }
                        }

                        button {
                            onclick : move |_| {
                                navigator.push(Router::Profile {});
                            }, "back"
                        }
                    }
                }
            }
        }
    }
}
