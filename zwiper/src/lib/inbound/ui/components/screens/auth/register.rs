use crate::{
    domain::error::UserFacing,
    inbound::ui::components::interactions::swipe::{
        direction::Direction, onswipe::OnMouse, ontouch::OnTouch, state::SwipeState, VH_GAP,
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
    const MOVE_SWIPES: [Direction; 1] = [Direction::Down];
    const SUBMIT_SWIPE: Direction = Direction::Up;

    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_attempted = use_signal(|| false);
    let mut is_loading = use_signal(|| false);

    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);
    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let maybe_submit = move || async move {
        if swipe_state.read().previous_swipe == Some(SUBMIT_SWIPE) {
            submit_attempted.set(true);
            is_loading.set(true);

            if let Err(e) = Username::new(&username.read()) {
                username_error.set(Some(e.to_string()));
            } else {
                username_error.set(None)
            }

            if let Err(e) = EmailAddress::from_str(&email.read()) {
                email_error.set(Some(e.to_string()));
            } else {
                email_error.set(None);
            }

            if let Err(e) = Password::new(&password.read()) {
                password_error.set(Some(e.to_string()))
            } else {
                password_error.set(None);
            }

            if username_error.read().is_none()
                && email_error.read().is_none()
                && password_error.read().is_none()
            {
                match RawRegisterUser::new(&*username.read(), &*email.read(), &*password.read())
                    .map(HttpRegisterUser::from)
                {
                    Ok(request) => match auth_client.read().register(request).await {
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
            }
            is_loading.set(false);
        }
    };

    rsx! {
        div { class : "swipe-able",

            style : format!(
                "transform: translateY(calc({}px + {}vh + {}vh));
                transition: transform {}s;",
                swipe_state.read().dy().from_start,
                VH_GAP,
                swipe_state.read().position.y * VH_GAP,
                swipe_state.read().transition_seconds
            ),

            ontouchstart : move |e: Event<TouchData>| swipe_state.ontouchstart(e),
            ontouchmove : move |e: Event<TouchData>| swipe_state.ontouchmove(e),
            ontouchend : move |e: Event<TouchData>| {
                swipe_state.ontouchend(e, &MOVE_SWIPES);
                spawn(maybe_submit());
            },

            onmousedown : move |e: Event<MouseData>| swipe_state.onmousedown(e),
            onmousemove : move |e: Event<MouseData>| swipe_state.onmousemove(e),
            onmouseup : move |e: Event<MouseData>| {
                swipe_state.onmouseup(e, &MOVE_SWIPES);
                spawn(maybe_submit());
            },

            div { class : "form-container",
                h2 { "create profile ↓" }

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
                                    match Username::new(&event.value()) {
                                        Ok(_) => username_error.set(None),
                                        Err(e) => username_error.set(Some(e.to_string())),
                                    }
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
                                    match EmailAddress::from_str(&event.value()) {
                                        Ok(_) => email_error.set(None),
                                        Err(e) => email_error.set(Some(e.to_user_facing_string())),
                                    }
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
                                    match Password::new(&event.value()) {
                                        Ok(_) => password_error.set(None),
                                        Err(e) => password_error.set(Some(e.to_string())),
                                    }
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
