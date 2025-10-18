use crate::{
    domain::error::UserFacing,
    inbound::ui::components::interactions::swipe::{
        config::SwipeConfig, direction::Direction as Dir, state::SwipeState, Swipeable,
    },
    outbound::{
        client::auth::{register::Register as RegisterTrait, AuthClient},
        session::Persist,
    },
};
use dioxus::prelude::*;
use email_address::EmailAddress;
use std::str::FromStr;
use zwipe::{
    domain::{
        auth::models::{password::Password, register_user::RawRegisterUser, session::Session},
        user::models::username::Username,
    },
    inbound::http::handlers::auth::register_user::HttpRegisterUser,
};

#[component]
pub fn Register(swipe_state: Signal<SwipeState>) -> Element {
    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Down],
        submission_swipe: Some(Dir::Left),
        from_main_screen: Some(Dir::Down),
    };

    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let mut validate_username = move || {
        if let Err(e) = Username::new(&username.read()) {
            username_error.set(Some(e.to_string()));
        } else {
            username_error.set(None)
        }
    };

    let mut validate_email = move || {
        if let Err(e) = EmailAddress::from_str(&email.read()) {
            email_error.set(Some(e.to_user_facing_string()));
        } else {
            email_error.set(None);
        }
    };

    let mut validate_password = move || {
        if let Err(e) = Password::new(&password.read()) {
            password_error.set(Some(e.to_string()))
        } else {
            password_error.set(None);
        }
    };

    let no_errors = move || {
        username_error.read().is_none()
            && email_error.read().is_none()
            && password_error.read().is_none()
    };

    let create_register_request = move || {
        RawRegisterUser::new(&*username.read(), &*email.read(), &*password.read())
            .map(HttpRegisterUser::from)
    };

    use_effect({
        let mut s = swipe_state.clone();
        let c = swipe_config.clone();
        move || {
            if s.read().latest_swipe == c.submission_swipe && c.submission_swipe.is_some() {
                s.write().latest_swipe = None;
                submit_attempted.set(true);
                is_loading.set(true);

                validate_username();
                validate_email();
                validate_password();

                if no_errors() {
                    match create_register_request() {
                        Ok(request) => {
                            spawn(async move {
                                match auth_client.read().register(request).await {
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
                                }
                            });
                        }
                        Err(e) => submission_error.set(Some(e.to_string())),
                    }
                }
                is_loading.set(false);
            }
        }
    });

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "form-container",
                h2 { "← create profile" }

                form {
                    div { class : "form-group",
                        label { r#for : "username" }

                        if *submit_attempted.read() {
                            if let Some(error) = username_error.read().as_ref() {
                                div { class : "form-error",
                                    "{error}"
                                }
                            }
                        }

                        input {
                            id : "username",
                            r#type : "text",
                            placeholder : "username",
                            value : "{username}",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                username.set(event.value());
                                if *submit_attempted.read() {
                                    validate_username();
                                }
                            }
                        }
                    }

                    div { class : "form-group",
                        label { r#for : "email" }

                        if *submit_attempted.read() {
                            if let Some(error) = email_error.read().as_ref() {
                                div { class : "form-error",
                                    "{error}"
                                }
                            }
                        }

                        input {
                            id : "email",
                            r#type : "text",
                            placeholder : "email",
                            value : "{email}",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                email.set(event.value());
                                if *submit_attempted.read() {
                                    validate_email();
                                }
                            }
                        }
                    }

                    div { class : "form-group",
                        label { r#for : "password", "" }

                        if *submit_attempted.read() {
                            if let Some(error) = password_error.read().as_ref() {
                                div { class : "form-error",
                                    "{error}"
                                }
                            }
                        }

                        input {
                            id : "password",
                            r#type : "password",
                            placeholder : "password",
                            value : "{password}",
                            autocapitalize : "none",
                            spellcheck : "false",
                            oninput : move |event| {
                                password.set(event.value());
                                if *submit_attempted.read() {
                                    validate_password()
                                }
                            }
                        }
                    }

                    if *is_loading.read() {
                        div { class : "spinning-card" }
                    } else if let Some(error) = submission_error.read().as_deref() {
                        div { class: "form-error",
                            { format!("{}", error) }
                        }
                    }
                }
            }
        }
    }
}
