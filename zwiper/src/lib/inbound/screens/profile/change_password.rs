use crate::{
    inbound::components::{
        auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        fields::text_input::TextInput,
        success_messages::random_success_message,
    },
    outbound::client::{user::change_password::ClientChangePassword, ZwipeClient},
};
use dioxus::prelude::*;
use zwipe::{
    domain::auth::models::{password::Password, session::Session},
    inbound::http::{handlers::auth::change_password::HttpChangePassword, ApiError},
};

#[component]
pub fn ChangePassword() -> Element {
    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    // we do not validate current password on frontend
    // as to not lock them out of changing
    // if their current somehow defies policy
    let mut current_password = use_signal(String::new);

    let mut new_password = use_signal(String::new);
    let mut confirm_password = use_signal(String::new);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut validate_new_password = move || {
        if let Err(e) = Password::new(&new_password()) {
            password_error.set(Some(e.to_string()));
        } else if new_password().as_str() != confirm_password().as_str() {
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
        password_error().is_none()
    };

    let mut clear_inputs = move || {
        current_password.set(String::new());
        new_password.set(String::new());
        confirm_password.set(String::new());
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        if inputs_are_valid() {
            tracing::info!("change password");
            let request = HttpChangePassword::new(&current_password(), &new_password());
            spawn(async move {
                session.upkeep(auth_client);
                let Some(sesh) = session() else {
                    submission_error.set(Some(
                        ApiError::Unauthorized("session expired".to_string()).to_string(),
                    ));
                    return;
                };

                match auth_client().change_password(request, &sesh).await {
                    Ok(()) => {
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
            validate_new_password();
        }
    });

    rsx! {
        Bouncer {
            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center;",
                div { class : "container-sm",

                    h2 { class: "text-center mb-2 font-light tracking-wider", "change password" }

                    form { class: "flex-col text-center",

                        TextInput {
                            value: current_password,
                            id: "current_password".to_string(),
                            placeholder: "current password".to_string(),
                            input_type: "password".to_string(),
                        }

                        if submit_attempted() {
                            if let Some(error) = password_error() {
                                div { class : "message-error", "{error}" }
                            }
                        }

                        TextInput {
                            value: new_password,
                            id: "new_password".to_string(),
                            placeholder: "new password".to_string(),
                            input_type: "password".to_string(),
                        }

                        TextInput {
                            value: confirm_password,
                            id: "confirm_password".to_string(),
                            placeholder: "confirm new".to_string(),
                            input_type: "password".to_string(),
                        }
                    }

                    if let Some(error) = submission_error() {
                        div { class: "message-error", "{error}" }
                    } else if let Some(success_message) = success_message() {
                        div { class: "message-success", {success_message} }
                    }
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "back"
                }
                button { class: "util-btn",
                    onclick : move |_| attempt_submit(),
                    "save changes"
                }
            }
        }
    }
}
