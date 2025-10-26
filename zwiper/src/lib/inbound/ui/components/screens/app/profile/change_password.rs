use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
            success_messages::random_success_message,
        },
        router::Router,
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
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    // we do not validate current password on frontend
    // as to not lock them out of changing
    // if their current somehow defies policy
    let mut current_password = use_signal(|| String::new());

    let mut new_password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
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
            let request = HttpChangePassword::new(&*current_password(), &*new_password());
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

                            if submit_attempted() {
                                if let Some(error) = password_error() {
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
                                    if submit_attempted() {
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
                                    if submit_attempted() {
                                        validate_new_password();
                                    }
                                }
                            }

                            button {
                                onclick : move |_| attempt_submit(),
                                "submit"
                            }

                            if let Some(error) = submission_error() {
                                div { class: "error", "{error}" }
                            } else if let Some(success_message) = success_message() {
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
