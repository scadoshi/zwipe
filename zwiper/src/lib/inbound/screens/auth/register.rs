//! New user registration screen.

use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::anonymous::record_anonymous_event;
use crate::{
    domain::error::UserFacing,
    inbound::{components::fields::text_input::TextInput, router::Router},
    outbound::{
        client::{ZwipeClient, auth::register::ClientRegister},
        session::Persist,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe::domain::auth::models::password::Password;
use zwipe_core::domain::{Email, auth::models::session::Session, logo, user::username::Username};
use zwipe_core::http::contracts::auth::HttpRegisterUser;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;

/// Registration form screen for creating new user accounts.
#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    // Funnel: register screen reached (once per mount; distinct-session
    // counting on the server dedupes revisits).
    use_hook(|| record_anonymous_event(auth_client, AnonymousEventKind::RegisterViewed));

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
            username_error.set(Some(e.to_string()));
        } else {
            username_error.set(None)
        }
    };

    let mut validate_email = move || {
        if let Err(e) = Email::new(email().trim()) {
            email_error.set(Some(e.to_user_facing_string()));
        } else {
            email_error.set(None);
        }
    };

    let mut validate_password = move || {
        if let Err(e) = Password::new(password()) {
            password_error.set(Some(e.to_string()))
        } else if password().as_str() != confirm_password().as_str() {
            password_error.set(Some("Passwords do not match".to_string()));
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

        validate_username();
        validate_email();
        validate_password();

        if !inputs_are_valid() {
            return;
        }
        is_loading.set(true);

        // Funnel: a validated submit is going to the server (success or not —
        // registration success itself lands in user_events).
        record_anonymous_event(auth_client, AnonymousEventKind::RegisterSubmitted);

        let email = email().trim().to_string();
        let request = HttpRegisterUser::new(&username(), &email, &password());
        spawn(async move {
            match auth_client().register(request).await {
                Ok(new_session) => {
                    new_session.infallible_save();
                    session.set(Some(new_session));
                    is_loading.set(false);
                    navigator.push(Router::Home {});
                }
                Err(e) => {
                    tracing::warn!("register failed: {e}");
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    is_loading.set(false);
                }
            }
        });
    };

    let mut username_touched = use_signal(|| false);
    let mut email_touched = use_signal(|| false);
    let mut password_touched = use_signal(|| false);

    use_effect(move || {
        let value = username();
        if !value.is_empty() && !username_touched() {
            username_touched.set(true);
        }
        if username_touched() || submit_attempted() {
            validate_username();
        }
    });

    use_effect(move || {
        let value = email();
        if !value.is_empty() && !email_touched() {
            email_touched.set(true);
        }
        if email_touched() || submit_attempted() {
            validate_email();
        }
    });

    use_effect(move || {
        let value = password();
        let confirm = confirm_password();
        if (!value.is_empty() || !confirm.is_empty()) && !password_touched() {
            password_touched.set(true);
        }
        if password_touched() || submit_attempted() {
            validate_password();
        }
    });

    rsx! {
        div { class: "screen",
            ScreenHeader { title: "Create profile" }
            div { class: "screen-content centered content-enter",
            div { class: "logo",  "{logo}" }
            div { class : "container-sm text-center",
                form { class: "flex-col",
                    TextInput {
                        value: username,
                        id: "username",
                        label: "Username",
                        placeholder: "Username",
                        error: username_error(),
                    }
                    TextInput {
                        value: email,
                        id: "email",
                        label: "Email",
                        placeholder: "Email",
                        input_type: "email",
                        error: email_error(),
                    }
                    TextInput {
                        value: password,
                        id: "password",
                        label: "Password",
                        placeholder: "Password",
                        input_type: "password",
                        error: password_error(),
                    }
                    TextInput {
                        value: confirm_password,
                        id: "confirm_password",
                        label: "Confirm password",
                        placeholder: "Confirm password",
                        input_type: "password",
                    }
                }
            }
        }
        div { class: "util-bar",
            button {
                class: "util-btn",
                disabled: is_loading(),
                onclick: move |_| navigator.go_back(),
                "Back to login"
            }
            button {
                class: "util-btn",
                disabled: is_loading(),
                onclick : move |_| attempt_submit(),
                if is_loading() { "Creating..." } else { "Create profile" }
            }
        }
    }
    }
}
