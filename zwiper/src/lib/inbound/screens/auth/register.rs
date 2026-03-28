//! New user registration screen.

use crate::{
    domain::error::UserFacing,
    inbound::{components::fields::text_input::TextInput, router::Router},
    outbound::{
        client::{auth::register::ClientRegister, ZwipeClient},
        session::Persist,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use email_address::EmailAddress;
use std::str::FromStr;
use std::time::Duration;
use zwipe::{
    domain::{
        auth::models::{password::Password, session::Session},
        logo,
        user::models::username::Username,
    },
    inbound::http::handlers::auth::register_user::HttpRegisterUser,
};

/// Registration form screen for creating new user accounts.
#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    let logo = logo::ZWIPE;

    let username = use_signal(String::new);
    let email = use_signal(String::new);
    let password = use_signal(String::new);
    let confirm_password = use_signal(String::new);

    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);
    let toast = use_toast();

    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);

    let mut validate_username = move || {
        if let Err(e) = Username::new(username()) {
            username_error.set(Some(e.to_string().to_lowercase()));
        } else {
            username_error.set(None)
        }
    };

    let mut validate_email = move || {
        if let Err(e) = EmailAddress::from_str(&email()) {
            email_error.set(Some(e.to_user_facing_string().to_lowercase()));
        } else {
            email_error.set(None);
        }
    };

    let mut validate_password = move || {
        if let Err(e) = Password::new(password()) {
            password_error.set(Some(e.to_string().to_lowercase()))
        } else if password().as_str() != confirm_password().as_str() {
            password_error.set(Some("passwords do not match".to_string()));
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
                        new_session.infallible_save();
                        session.set(Some(new_session));
                        navigator.push(Router::Home {});
                    }
                    Err(e) => toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    ),
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
        div { class: "screen",
            div { class: "screen-content centered",
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
                        label: "username",
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
                        label: "email",
                        placeholder: "email",
                    }
                    if submit_attempted() && let Some(error) = password_error() {
                        div { class : "message-error", "{error}" }
                    }
                    TextInput {
                        value: password,
                        id: "password",
                        label: "password",
                        placeholder: "password",
                        input_type: "password",
                    }
                    TextInput {
                        value: confirm_password,
                        id: "confirm_password",
                        label: "confirm password",
                        placeholder: "confirm password",
                        input_type: "password",
                    }
                    if is_loading() {
                        div { class : "spinner" }
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
}
