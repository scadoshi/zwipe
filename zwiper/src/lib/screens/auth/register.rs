use std::str::FromStr;

use crate::swipe::{self, Direction as Dir, OnMouse, OnTouch, VH_GAP};
use dioxus::prelude::*;
use email_address::EmailAddress;
use zwipe::domain::{auth::models::password::Password, user::models::Username};

pub trait EmailErrorAdjustment {
    fn adjusted_string(&self) -> String;
}

impl EmailErrorAdjustment for email_address::Error {
    fn adjusted_string(&self) -> String {
        match self {
            email_address::Error::InvalidCharacter => "invalid character in email".to_string(),
            email_address::Error::MissingSeparator => "missing @ symbol".to_string(),
            email_address::Error::LocalPartEmpty => "missing text before @".to_string(),
            email_address::Error::LocalPartTooLong => "text before @ is too long".to_string(),
            email_address::Error::DomainEmpty => "missing domain after @".to_string(),
            email_address::Error::DomainTooLong => "domain is too long".to_string(),
            email_address::Error::SubDomainEmpty => "empty part in domain".to_string(),
            email_address::Error::SubDomainTooLong => {
                "a part of the domain is too long".to_string()
            }
            email_address::Error::DomainTooFew => "domain must have at least one dot".to_string(),
            email_address::Error::DomainInvalidSeparator => {
                "invalid dot placement in domain".to_string()
            }
            email_address::Error::UnbalancedQuotes => "unbalanced quotes in email".to_string(),
            email_address::Error::InvalidComment => "invalid comment in email".to_string(),
            email_address::Error::InvalidIPAddress => "invalid ip address in domain".to_string(),
            email_address::Error::UnsupportedDomainLiteral => {
                "domain literal not supported".to_string()
            }
            email_address::Error::UnsupportedDisplayName => {
                "display name not supported".to_string()
            }
            email_address::Error::MissingDisplayName => "missing display name".to_string(),
            email_address::Error::MissingEndBracket => "missing closing bracket".to_string(),
        }
    }
}

#[component]
pub fn Register(swipe_state: Signal<swipe::State>) -> Element {
    const MOVE_SWIPES: [Dir; 1] = [Dir::Down];
    const SUBMIT_SWIPE: Dir = Dir::Up;

    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_attempted: Signal<bool> = use_signal(|| false);

    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);

    let mut maybe_submit = move || {
        if swipe_state.read().previous_swipe == Some(SUBMIT_SWIPE) {
            submit_attempted.set(true);

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
                println!("please make my account");
            }
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
                maybe_submit();
            },

            onmousedown : move |e: Event<MouseData>| swipe_state.onmousedown(e),
            onmousemove : move |e: Event<MouseData>| swipe_state.onmousemove(e),
            onmouseup : move |e: Event<MouseData>| {
                swipe_state.onmouseup(e, &MOVE_SWIPES);
                maybe_submit();
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
                                        Err(e) => email_error.set(Some(e.adjusted_string())),
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
                }
            }
        }
    }
}
