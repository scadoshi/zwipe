use std::str::FromStr;

use crate::{
    client::auth::{
        login::{Login as LoginTrait, LoginError},
        AuthClient,
    },
    swipe::{self, Direction as Dir, OnMouse, OnTouch, VH_GAP},
};
use dioxus::prelude::*;
use email_address::EmailAddress;
use zwipe::{
    domain::{
        auth::models::{authenticate_user::AuthenticateUser, password::Password},
        user::models::Username,
    },
    inbound::http::handlers::auth::authenticate_user::HttpAuthenticateUser,
};

#[component]
pub fn Login(swipe_state: Signal<swipe::State>) -> Element {
    const MOVE_SWIPES: [Dir; 1] = [Dir::Up];
    const SUBMIT_SWIPE: Dir = Dir::Down;

    let auth_client = use_signal(|| AuthClient::new());

    let mut username_or_email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_attempted: Signal<bool> = use_signal(|| false);
    let mut is_loading = use_signal(|| false);
    let mut submission_error: Signal<Option<String>> = use_signal(|| None);

    let valid_credentials = move || {
        let valid_username_or_email = Username::new(&username_or_email.read()).is_ok()
            || EmailAddress::from_str(&username_or_email.read()).is_ok();
        let valid_password = Password::new(&password.read()).is_ok();
        if valid_username_or_email && valid_password {
            true
        } else {
            false
        }
    };

    let maybe_submit = move || async move {
        if swipe_state.read().previous_swipe == Some(SUBMIT_SWIPE) {
            submit_attempted.set(true);
            is_loading.set(true);

            if valid_credentials() {
                match AuthenticateUser::new(&*username_or_email.read(), &*password.read())
                    .map(HttpAuthenticateUser::from)
                {
                    Ok(request) => match auth_client.read().authenticate_user(request).await {
                        Ok(s) => {
                            submission_error.set(None);
                            println!("session => {:#?}", s);
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
        div { class : "swipe-able",

            style : format!(
                "transform: translateY(calc({}px - {}vh + {}vh));
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
