use crate::{
    inbound::ui::components::interactions::swipe::{
        config::SwipeConfig, direction::Direction as Dir, state::SwipeState, Swipeable,
    },
    outbound::{
        client::auth::{
            login::{Login as LoginTrait, LoginError},
            AuthClient,
        },
        session::Persist,
    },
};
use dioxus::prelude::*;
use email_address::EmailAddress;
use std::str::FromStr;
use zwipe::{
    domain::{
        auth::models::{authenticate_user::AuthenticateUser, password::Password, session::Session},
        user::models::username::Username,
    },
    inbound::http::handlers::auth::authenticate_user::HttpAuthenticateUser,
};

#[component]
pub fn Login(swipe_state: Signal<SwipeState>) -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<AuthClient> = use_context();

    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Up],
        submission_swipe: Some(Dir::Right),
        from_main_screen: Some(Dir::Up),
    };

    let mut username_or_email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_attempted: Signal<bool> = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let valid_credentials = move || {
        (Username::new(&username_or_email.read()).is_ok()
            || EmailAddress::from_str(&username_or_email.read()).is_ok())
            && Password::new(&password.read()).is_ok()
    };

    let create_login_request = move || {
        AuthenticateUser::new(&*username_or_email.read(), &*password.read())
            .map(HttpAuthenticateUser::from)
    };

    use_effect({
        let mut s = swipe_state.clone();
        let c = swipe_config.clone();
        move || {
            if s.read().latest_swipe == c.submission_swipe && c.submission_swipe.is_some() {
                s.write().latest_swipe = None;
                submit_attempted.set(true);
                is_loading.set(true);

                if valid_credentials() {
                    match create_login_request() {
                        Ok(request) => {
                            spawn(async move {
                                match auth_client.read().authenticate_user(request).await {
                                    Ok(new_session) => {
                                        submission_error.set(None);

                                        if let Err(e) = new_session.save() {
                                            tracing::error!("failed to save session: {e}");
                                        }

                                        session.set(Some(new_session));
                                    }
                                    Err(e) => submission_error.set(Some(e.to_string())),
                                }
                            });
                        }
                        Err(e) => submission_error.set(Some(e.to_string())),
                    }
                } else {
                    submission_error.set(Some(LoginError::Unauthorized.to_string()));
                }

                is_loading.set(false);
            }
        }
    });

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

                h2 { "login →"}
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
