use crate::{
    domain::error::UserFacing,
    inbound::ui::{
        components::interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        router::Router,
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
        auth::models::{password::Password, session::Session},
        logo,
        user::models::username::Username,
    },
    inbound::http::handlers::auth::register_user::HttpRegisterUser,
};

#[component]
pub fn Register() -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let mut session: Signal<Option<Session>> = use_context();
    let auth_client: Signal<AuthClient> = use_context();

    let logo = logo::logo();

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

    let mut inputs_are_valid = move || {
        validate_username();
        validate_email();
        validate_password();
        username_error.read().is_none()
            && email_error.read().is_none()
            && password_error.read().is_none()
    };

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class: "logo",  "{logo}" }
            div { class : "form-container",
            form {
                div { class : "form-group",
                    label { r#for : "username" }

                    if *submit_attempted.read() {
                        if let Some(error) = username_error.read().as_ref() {
                            div { class : "error",
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
                    label { r#for : "email" }

                    if *submit_attempted.read() {
                        if let Some(error) = email_error.read().as_ref() {
                            div { class : "error",
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

                    label { r#for : "password", "" }

                    if *submit_attempted.read() {
                        if let Some(error) = password_error.read().as_ref() {
                            div { class : "error",
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

                    button {
                        onclick : move |_| {
                            submit_attempted.set(true);
                            is_loading.set(true);

                            validate_username();
                            validate_email();
                            validate_password();

                            if inputs_are_valid() {
                                let request = HttpRegisterUser::new(&*username.read(), &*email.read(), &*password.read());
                                spawn(async move {
                                    match auth_client.read().register(request).await {
                                        Ok(new_session) => {
                                            submission_error.set(None);

                                            if let Err(e) = new_session.save() {
                                                tracing::error!("failed to save session: {e}");
                                            }

                                            session.set(Some(new_session));
                                            navigator.push(Router::Home {});
                                        }
                                        Err(e) => submission_error.set(Some(e.to_string())),
                                    }
                                });
                            }
                            is_loading.set(false);
                        }, "create profile"
                    }
                    button {
                        onclick : move |_| {
                            navigator.push(Router::Login {});
                        }, "back"
                    }

                }

                if *is_loading.read() {
                    div { class : "spinning-card" }
                } else if let Some(error) = submission_error.read().as_deref() {
                    div { class: "error",
                        { format!("{}", error) }
                    }
                }
            }
            }
        }
    }
}
