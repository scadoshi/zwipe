//! New user registration screen.

use components::{ActionBar, Button, ButtonVariant, TOAST_NORMAL, TextInput};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::Email;
use zwipe_core::domain::auth::password::validate;
use zwipe_core::domain::user::username::Username;
use zwipe_core::http::contracts::auth::HttpRegisterUser;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;

use crate::domain::error::UserFacing;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::home_hero::HomeHero;
use crate::inbound::components::info_button;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::anonymous;
use crate::inbound::screens::auth::login::LoginPrefill;

/// Registration form screen for creating new user accounts.
#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();

    let LoginPrefill(mut login_prefill) = use_context();
    let auth_client: Signal<ZwipeClient> = use_context();

    // Funnel: register screen reached (once per mount; distinct-session
    // counting on the server dedupes revisits).
    use_hook(|| anonymous::record_anonymous_event(auth_client, AnonymousEventKind::RegisterViewed));

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
        if let Err(e) = validate(&password()) {
            password_error.set(Some(e.to_string()))
        } else if password().as_str() != confirm_password().as_str()
            // Mid-typing the confirmation can't match yet; judge it once it
            // is as long as the password, or on submit.
            && (submit_attempted() || confirm_password().chars().count() >= password().chars().count())
        {
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

        // Funnel: a validated submit is going to the server (success or not,
        // registration success itself lands in user_events).
        anonymous::record_anonymous_event(auth_client, AnonymousEventKind::RegisterSubmitted);

        let email = email().trim().to_string();
        let username = username();
        let request = HttpRegisterUser::new(&username, &email, &password());
        spawn(async move {
            // The same answer whether or not the email already has an
            // account, so this screen can't tell either and says the same.
            match auth_client().register(request).await {
                Ok(_) => {
                    is_loading.set(false);
                    login_prefill.set(Some(username));
                    toast.info(
                        "Check your inbox to verify your email, then log in".to_string(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                    navigator.go_back();
                }
                Err(e) => {
                    tracing::warn!("register failed: {e}");
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(TOAST_NORMAL),
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
            div { class: "screen-content content-enter",
            HomeHero {}
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
                        on_info: info_button::open_topic(HintTopic::Password),
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
        ActionBar {
            Button {
                variant: ButtonVariant::Action,
                disabled: is_loading(),
                onclick: move |_| navigator.go_back(),
                "Back to login"
            }
            Button {
                variant: ButtonVariant::Action,
                disabled: is_loading(),
                onclick : move |_| attempt_submit(),
                if is_loading() { "Creating..." } else { "Create profile" }
            }
        }
    }
    }
}
