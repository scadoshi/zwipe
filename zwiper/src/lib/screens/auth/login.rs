use std::str::FromStr;

use crate::{
    http::auth::{authenticate_user, validate_authenticate_user, AuthClient},
    swipe::{self, Direction as Dir, OnMouse, OnTouch, VH_GAP},
};
use dioxus::prelude::*;
use email_address::EmailAddress;
use zwipe::domain::{auth::models::password::Password, user::models::Username};

#[component]
pub fn Login(swipe_state: Signal<swipe::State>) -> Element {
    const MOVE_SWIPES: [Dir; 1] = [Dir::Up];
    const SUBMIT_SWIPE: Dir = Dir::Down;

    let auth_client = use_signal(|| AuthClient::default());

    let mut username_or_email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_attempted: Signal<bool> = use_signal(|| false);

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

            if valid_credentials() {
                // let auth_client_clone = auth_client.read().clone();
                // let's try this without cloning for a sec
                match validate_authenticate_user(&*username_or_email.read(), &*password.read()) {
                    Ok(request) => match authenticate_user(request, &auth_client.read()).await {
                        Ok(s) => {
                            println!("authenticated user => {:#?}", s.user);
                            println!("token => {:?}", s.token)
                        }
                        Err(e) => submission_error.set(Some(e.to_string())),
                    },
                    Err(e) => submission_error.set(Some(e.to_string())),
                }
            }
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

                h2 { "login ↑"}

                if *submit_attempted.read() && !valid_credentials() {
                    div { class : "form-error",
                        "invalid credentials"
                    }
                }

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

                    if let Some(error) = submission_error.read().as_deref() {
                        div { class: "form-error",
                                { format!("{}", error) }
                        }
                    }
                }
            }
        }
    }
}
