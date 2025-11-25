use crate::domain::error::UserFacing;
use crate::inbound::ui::components::fields::text_input::TextInput;
use crate::inbound::ui::components::interactions::swipe::Swipeable;
use crate::inbound::ui::components::{
    auth::{bouncer::Bouncer, session_upkeep::Upkeep},
    interactions::swipe::{config::SwipeConfig, state::SwipeState},
    success_messages::random_success_message,
};
use crate::outbound::client::user::change_email::ClientChangeEmail;
use crate::outbound::client::ZwipeClient;
use dioxus::prelude::*;
use email_address::EmailAddress;
use std::str::FromStr;
use zwipe::{
    domain::auth::models::{password::Password, session::Session},
    inbound::http::handlers::auth::change_email::HttpChangeEmail,
    inbound::http::ApiError,
};

#[component]
pub fn ChangeEmail() -> Element {
    let swipe_state = use_signal(SwipeState::new);
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let mut new_email = use_signal(String::new);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_email = move || {
        if let Err(e) = EmailAddress::from_str(&new_email()) {
            email_error.set(Some(e.to_user_facing_string()));
        } else {
            email_error.set(None)
        }
    };

    let mut password = use_signal(String::new);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_password = move || {
        if Password::new(&password()).is_err() {
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
        email_error().is_none() && password_error().is_none()
    };

    let mut clear_inputs = move || {
        new_email.set(String::new());
        password.set(String::new());
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        if inputs_are_valid() {
            tracing::info!("change email to {}", new_email());
            let request = HttpChangeEmail::new(&new_email(), &password());
            spawn(async move {
                session.upkeep(auth_client);
                let Some(mut sesh) = session() else {
                    submission_error.set(Some(
                        ApiError::Unauthorized("session expired".to_string()).to_string(),
                    ));
                    return;
                };

                match auth_client().change_email(request, &sesh).await {
                    Ok(updated_user) => {
                        submission_error.set(None);
                        sesh.user.email = updated_user.email;
                        session.set(Some(sesh));
                        success_message.set(Some(random_success_message()));
                        clear_inputs();
                        submit_attempted.set(false);
                    }
                    Err(e) => submission_error.set(Some(e.to_string())),
                }
            });
        } else {
            submission_error.set(Some("invalid input".to_string()));
        }
    };

    use_effect(move || {
        if submit_attempted() {
            validate_email();
            validate_password();
        }
    });

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "container-sm",

                    h2 { class: "text-center mb-2 font-light tracking-wider", "change email" }

                    form { class: "flex-col text-center",

                        if submit_attempted() {
                            if let Some(error) = email_error() {
                                div { class : "message-error", "{error}" }
                            }
                        }

                        TextInput {
                            value: new_email,
                            id: "new_email".to_string(),
                            placeholder: "new email".to_string(),
                        }

                        if submit_attempted() {
                            if let Some(error) = password_error() {
                                div { class : "message-error", "{error}" }
                            }
                        }

                        TextInput {
                            value: password,
                            id: "password".to_string(),
                            placeholder: "password".to_string(),
                            input_type: "password".to_string(),
                        }

                        button { class: "btn",
                            onclick : move |_| attempt_submit(),
                            "submit"
                        }

                        button { class: "btn",
                            onclick : move |_| {
                                navigator.go_back();
                            }, "back"
                        }
                    }

                    if let Some(error) = submission_error() {
                        div { class: "message-error", "{error}" }
                    } else if let Some(success_message) = success_message() {
                        div { class: "message-success", {success_message} }
                    }
                }
            }
        }
    }
}
