use crate::{
    inbound::{
        components::fields::text_input::TextInput,
        router::Router,
    },
    outbound::{
        client::{auth::login::ClientLogin, ZwipeClient},
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
    inbound::http::{handlers::auth::authenticate_user::HttpAuthenticateUser, ApiError},
};

#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let logo = logo::ZWIPE;

    let username_or_email = use_signal(String::new);
    let password = use_signal(String::new);

    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let inputs_are_valid = move || {
        (Username::new(&username_or_email()).is_ok()
            || EmailAddress::from_str(&username_or_email()).is_ok())
            && Password::new(&password()).is_ok()
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        is_loading.set(true);
        if inputs_are_valid() {
            let request = HttpAuthenticateUser::new(&username_or_email(), &password());
            spawn(async move {
                match auth_client().authenticate_user(request).await {
                    Ok(new_session) => {
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
        div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
            style: "width: 100vw; justify-content: center;",
            div { class: "logo",  "{logo}" }
            div { class : "container-sm text-center",

            form { class: "flex-col",

                TextInput {
                    value: username_or_email,
                    id: "identity".to_string(),
                    placeholder: "username or email".to_string(),
                }

                TextInput {
                    value: password,
                    id: "password".to_string(),
                    placeholder: "password".to_string(),
                    input_type: "password".to_string(),
                }

                button { class: "btn",
                    onclick : move |_| attempt_submit(),
                    "login"
                }

                if is_loading() {
                    div { class : "spinner" }
                } else if let Some(error) = submission_error() {
                    div { class: "message-error",
                        { error }
                    }
                }

                button { class: "btn",
                    onclick : move |_| {
                        navigator.push(Router::Register {});
                    }, "create profile"
                }
            }
            }
        }
    }
}
