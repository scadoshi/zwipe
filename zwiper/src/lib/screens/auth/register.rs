use std::str::FromStr;

use dioxus::prelude::*;
use email_address::EmailAddress;
use zwipe::domain::{auth::models::password::Password, user::models::Username};

use crate::{screens::Screen, swipe};

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
    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_clicked: Signal<bool> = use_signal(|| false);

    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div { class : "swipe-able",

            style : format!(
                "transform: translateY(calc({}px + 100vh));
                transition: transform {}s;",
                swipe_state.read().dy().from_start,
                swipe_state.read().transition_seconds
            ),

            div { class : "form-container",
                h2 { "create profile" }

                form {
                    onsubmit : move |_| {
                        submit_clicked.set(true);
                        match Username::new(&username.read()) {
                            Ok(_) => username_error.set(None),
                            Err(e) => username_error.set(Some(e.to_string())),
                        }

                        match EmailAddress::from_str(&email.read()) {
                            Ok(_) => email_error.set(None),
                            Err(e) => {
                                email_error.set(Some(e.adjusted_string()));
                            }
                        }

                        match Password::new(&password.read()) {
                            Ok(_) => password_error.set(None),
                            Err(e) => password_error.set(Some(e.to_string())),
                        }

                        if username_error.read().is_none() && email_error.read().is_none() && password_error.read().is_none() {
                            println!("please make my account");
                        }
                    },

                    div { class : "form-group",
                        label { r#for : "username" }

                        if *submit_clicked.read() {
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
                                if *submit_clicked.read() {
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

                        if *submit_clicked.read() {
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
                                if *submit_clicked.read() {
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

                        if *submit_clicked.read() {
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
                                if *submit_clicked.read() {
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
