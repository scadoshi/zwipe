use std::str::FromStr;

use crate::swipe::{self, Direction as Dir, OnMouse, OnTouch, VH_GAP};
use dioxus::prelude::*;
use email_address::EmailAddress;
use zwipe::domain::{auth::models::password::Password, user::models::Username};

#[component]
pub fn Login(swipe_state: Signal<swipe::State>) -> Element {
    let mut username_or_email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut invalid_credentials = use_signal(|| false);

    let mut attempt_submit = move || {
        let valid_identifier = match (
            Username::new(&username_or_email.read()),
            EmailAddress::from_str(&username_or_email.read()),
        ) {
            (Ok(_), _) | (_, Ok(_)) => true,
            (Err(_), Err(_)) => false,
        };

        let valid_password = Password::new(&password.read()).is_ok();

        if !valid_identifier || !valid_password {
            invalid_credentials.set(true);
            return;
        }

        invalid_credentials.set(false);
        println!("please log me in");
    };

    const ALLOWED_DIRECTIONS: [Dir; 1] = [Dir::Up];

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
                swipe_state.ontouchend(e, &ALLOWED_DIRECTIONS);
                if swipe_state.read().previous_swipe == Some(Dir::Down) {
                    attempt_submit();
                }
            },

            onmousedown : move |e: Event<MouseData>| swipe_state.onmousedown(e),
            onmousemove : move |e: Event<MouseData>| swipe_state.onmousemove(e),
            onmouseup : move |e: Event<MouseData>| {
                swipe_state.onmouseup(e, &ALLOWED_DIRECTIONS);
                if swipe_state.read().previous_swipe == Some(Dir::Down) {
                    attempt_submit()
                }
            },

            div { class : "form-container",

                h2 { "login ↑"}

                if *invalid_credentials.read() {
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
                }
            }
        }
    }
}
