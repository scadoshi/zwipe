use crate::inbound::ui::components::interactions::swipe::config::SwipeConfig;
use crate::inbound::ui::components::interactions::swipe::direction::Direction;
use crate::inbound::ui::components::interactions::swipe::Swipeable;
use crate::outbound::client::auth::AuthClient;
use crate::{
    inbound::ui::components::interactions::swipe::state::SwipeState,
    outbound::{
        client::auth::login::{Login as LoginTrait, LoginError},
        session::Persist,
    },
};
use dioxus::prelude::*;
use email_address::EmailAddress;
use std::str::FromStr;
use zwipe::domain::auth::models::session::Session;
use zwipe::{
    domain::{
        auth::models::{authenticate_user::AuthenticateUser, password::Password},
        user::models::username::Username,
    },
    inbound::http::handlers::auth::authenticate_user::HttpAuthenticateUser,
};

#[component]
pub fn Login(swipe_state: Signal<SwipeState>) -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<AuthClient> = use_context();

    let swipe_config = SwipeConfig::new(
        vec![Direction::Up],
        Some(Direction::Down),
        Some(Direction::Up),
    );

    let mut username_or_email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_attempted: Signal<bool> = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let valid_credentials = move || -> bool {
        let valid_username_or_email = Username::new(&username_or_email.read()).is_ok()
            || EmailAddress::from_str(&username_or_email.read()).is_ok();
        let valid_password = Password::new(&password.read()).is_ok();
        if valid_username_or_email && valid_password {
            true
        } else {
            false
        }
    };

    let maybe_submit = move |swipe_config: SwipeConfig| async move {
        if swipe_state.read().latest_swipe == swipe_config.submission_swipe
            && swipe_config.submission_swipe.is_some()
        {
            submit_attempted.set(true);
            is_loading.set(true);

            if valid_credentials() {
                match AuthenticateUser::new(&*username_or_email.read(), &*password.read())
                    .map(HttpAuthenticateUser::from)
                {
                    Ok(request) => match auth_client.read().authenticate_user(request).await {
                        Ok(new_session) => {
                            submission_error.set(None);

                            if let Err(e) = new_session.save() {
                                tracing::error!("failed to save session: {e}");
                            } else {
                                tracing::info!("saved session successfully");
                            }

                            session.set(Some(new_session));
                        }
                        Err(e) => submission_error.set(Some(e.to_string())),
                    },
                    Err(e) => submission_error.set(Some(e.to_string())),
                }
            } else {
                submission_error.set(Some(LoginError::Unauthorized.to_string()));
            }
            is_loading.set(false);
        }
    };

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "form-container",

                if *is_loading.read() {
                    div { class : "spinning-card" }
                } else if let Some(error) = submission_error.read().as_deref() {
                    div { class: "form-error",
                        { format!("{}", error) }
                    }
                }

                h2 { "login ↑"}
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
                    }
                    div { class : "form-group",
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
                    }
                }
            }
        }
    }
}
