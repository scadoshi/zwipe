use crate::{
    inbound::ui::components::{
        auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        fields::text_input::TextInput,
        interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        success_messages::random_success_message,
    },
    outbound::client::{user::change_username::ClientChangeUsername, ZwipeClient},
};
use dioxus::prelude::*;
use zwipe::{
    domain::{
        auth::models::{password::Password, session::Session},
        user::models::username::Username,
    },
    inbound::http::{handlers::auth::change_username::HttpChangeUsername, ApiError},
};

#[component]
pub fn ChangeUsername() -> Element {
    let swipe_state = use_signal(SwipeState::new);
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let mut new_username = use_signal(String::new);
    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_username = move || {
        if let Err(e) = Username::new(&new_username()) {
            username_error.set(Some(e.to_string()));
        } else {
            username_error.set(None)
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
            spawn(async move {
                session.upkeep(auth_client);
                let Some(mut sesh) = session() else {
                    submission_error.set(Some(
                        ApiError::Unauthorized("session expired".to_string()).to_string(),
                    ));
                    return;
                };

                match auth_client().change_username(request, &sesh).await {
                    Ok(updated_user) => {
                        sesh.user.username = updated_user.username;
                        session.set(Some(sesh));
                        success_message.set(Some(random_success_message()));
                        submission_error.set(None);
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
            validate_username();
            validate_password();
        }
    });

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "container-sm",

                    h2 { class: "text-center mb-2 font-light tracking-wider", "change username" }

                    form { class: "flex-col text-center",

                        if submit_attempted() {
                            if let Some(error) = username_error() {
                                div { class : "message-error", "{error}" }
                            }
                        }

                        TextInput {
                            value: new_username,
                            id: "new_username".to_string(),
                            placeholder: "new username".to_string(),
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
