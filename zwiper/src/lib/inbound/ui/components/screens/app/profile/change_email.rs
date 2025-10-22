use crate::domain::error::UserFacing;
use crate::inbound::ui::{
    components::{
        auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        interactions::swipe::{config::SwipeConfig, state::SwipeState},
        screens::app::profile::Swipeable,
        success_messages::random_success_message,
    },
    router::Router,
};
use crate::outbound::client::auth::{
    change_email::{AuthClientChangeEmail, ChangeEmailError},
    AuthClient,
};
use dioxus::prelude::*;
use email_address::EmailAddress;
use std::str::FromStr;
use zwipe::{
    domain::auth::models::{password::Password, session::Session},
    inbound::http::handlers::auth::change_email::HttpChangeEmail,
};

#[component]
pub fn ChangeEmail() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<AuthClient> = use_context();

    let mut new_email = use_signal(|| String::new());
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_email = move || {
        if let Err(e) = EmailAddress::from_str(&new_email.read()) {
            email_error.set(Some(e.to_user_facing_string()));
        } else {
            email_error.set(None)
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
        validate_email();
        validate_password();
        email_error.read().is_none() && password_error.read().is_none()
    };

    let mut clear_inputs = move || {
        new_email.set(String::new());
        password.set(String::new());
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        if inputs_are_valid() {
            tracing::info!("change email to {}", new_email.read());
            let request = HttpChangeEmail::new(&*new_email.read(), &*password.read());
            spawn(async move {
                session.upkeep(auth_client);
                let Some(mut sesh) = session.read().clone() else {
                    submission_error.set(Some(ChangeEmailError::SessionExpired.to_string()));
                    return;
                };

                match auth_client.read().change_email(request, &sesh).await {
                    Ok(updated_user) => {
                        submission_error.set(None);
                        sesh.user.email = updated_user.email;
                        session.set(Some(sesh));
                        success_message.set(Some(random_success_message()));
                        clear_inputs();
                    }
                    Err(e) => submission_error.set(Some(e.to_string())),
                }
            });
        } else {
            submission_error.set(Some("invalid input".to_string()));
        }
    };

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "form-container",

                    h2 { "change email" }

                    form {
                        div { class : "form-group",

                            if *submit_attempted.read() {
                                if let Some(error) = email_error.read().as_ref() {
                                    div { class : "error", "{error}" }
                                }
                            }

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
                                onclick : move |_| attempt_submit(),
                                "submit"
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
}
