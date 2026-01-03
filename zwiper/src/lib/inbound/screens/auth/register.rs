use crate::{
    domain::error::UserFacing,
    inbound::{components::fields::text_input::TextInput, router::Router},
    outbound::{
        client::{auth::register::ClientRegister, ZwipeClient},
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
    inbound::http::handlers::auth::register_user::HttpRegisterUser,
};

#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let logo = logo::ZWIPE;

    let username = use_signal(String::new);
    let email = use_signal(String::new);
    let password = use_signal(String::new);

    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let mut validate_username = move || {
        if let Err(e) = Username::new(&username()) {
            username_error.set(Some(e.to_string()));
        } else {
            username_error.set(None)
        }
    };

    let mut validate_email = move || {
        if let Err(e) = EmailAddress::from_str(&email()) {
            email_error.set(Some(e.to_user_facing_string()));
        } else {
            email_error.set(None);
        }
    };

    let mut validate_password = move || {
        if let Err(e) = Password::new(&password()) {
            password_error.set(Some(e.to_string()))
        } else {
            password_error.set(None);
        }
    };

    let mut inputs_are_valid = move || {
        validate_username();
        validate_email();
        validate_password();
        username_error().is_none() && email_error().is_none() && password_error().is_none()
    };

    let mut attempt_submit = move || {
        submit_attempted.set(true);
        is_loading.set(true);

        validate_username();
        validate_email();
        validate_password();

        if inputs_are_valid() {
            let request = HttpRegisterUser::new(&username(), &email(), &password());
            spawn(async move {
                match auth_client().register(request).await {
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
        }
        is_loading.set(false);
    };

    use_effect(move || {
        if submit_attempted() {
            validate_username();
            validate_email();
            validate_password();
        }
    });

    rsx! {
        div { class: "sticky top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
            style: "width: 100vw; justify-content: center;",
            div { class: "logo",  "{logo}" }
            div { class : "container-sm text-center",
                form { class: "flex-col",
                    if submit_attempted() {
                        if let Some(error) = username_error() {
                            div { class : "message-error", "{error}" }
                        }
                    }
                    TextInput {
                        value: username,
                        id: "username",
                        placeholder: "username",
                     }
                    if submit_attempted() {
                        if let Some(error) = email_error() {
                            div { class : "message-error", "{error}" }
                        }
                    }
                    TextInput {
                        value: email,
                        id: "email",
                        placeholder: "email",
                    }
                    if submit_attempted() && let Some(error) = password_error() {
                        div { class : "message-error", "{error}" }
                    }
                    TextInput {
                        value: password,
                        id: "password",
                        placeholder: "password",
                        input_type: "password",
                    }
                    if is_loading() {
                        div { class : "spinner" }
                    } else if let Some(error) = submission_error() {
                        div { class: "message-error",
                            { error }
                        }
                    }
                }
            }
        }
        div { class: "util-bar",
            button {
                class: "util-btn",
                onclick: move |_| navigator.go_back(),
                "back to login"
            }
            button {
                class: "util-btn",
                onclick : move |_| attempt_submit(),
                "create profile"
            }
        }
    }
}
