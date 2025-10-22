use crate::{
    inbound::ui::{
        components::interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        router::Router,
    },
    outbound::{
        client::{
            auth::{login::AuthClientLogin, AuthClient},
            error::ApiError,
        },
        session::Persist,
    },
};
use dioxus::prelude::*;
use email_address::EmailAddress;
use std::str::FromStr;
use zwipe::{
    domain::{
        auth::models::{password::Password, session::Session},
        logo,
        user::models::username::Username,
    },
    inbound::http::handlers::auth::authenticate_user::HttpAuthenticateUser,
};

#[component]
pub fn Login() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<AuthClient> = use_context();

    let logo = logo::logo();

    let mut username_or_email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let inputs_are_valid = move || {
        (Username::new(&username_or_email.read()).is_ok()
            || EmailAddress::from_str(&username_or_email.read()).is_ok())
            && Password::new(&password.read()).is_ok()
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        is_loading.set(true);
        if inputs_are_valid() {
            let request = HttpAuthenticateUser::new(&*username_or_email.read(), &*password.read());
            spawn(async move {
                match auth_client.read().authenticate_user(request).await {
                    Ok(new_session) => {
                        // tracing::info!("session={:?}", new_session);
                        submission_error.set(None);
                        new_session.infallible_save();
                        session.set(Some(new_session));
                        navigator.push(Router::Home {});
                    }
                    Err(e) => submission_error.set(Some(e.to_string())),
                }
            });
        } else {
            submission_error.set(Some(
                ApiError::Unauthorized("invalid credentials".to_string()).to_string(),
            ));
        }
        is_loading.set(false);
    };

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class: "logo",  "{logo}" }
            div { class : "form-container",

                form {
                    div { class : "form-group",
                        label { r#for: "identity" }
                        input {
                            id : "identity",
                            r#type : "text",
                            placeholder : "username or email",
                            value : "{username_or_email}",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput: move |event| {
                                username_or_email.set(event.value());
                            }
                        }
                        label { r#for : "password", "" }
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
                            onclick : move |_| attempt_submit(),
                            "login"
                        }

                        if *is_loading.read() {
                            div { class : "spinning-card" }
                        } else if let Some(error) = submission_error.read().as_deref() {
                            div { class: "error",
                                { format!("{}", error) }
                            }
                        }

                        button {
                            onclick : move |_| {
                                navigator.push(Router::Register {});
                            }, "create profile"
                        }
                    }
                }
            }
        }
    }
}
