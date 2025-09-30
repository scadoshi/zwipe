use std::str::FromStr;

use dioxus::prelude::*;
use email_address::EmailAddress;
use zwipe::domain::{auth::models::password::Password, user::models::Username};

use crate::routing::Route;

#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();
    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut submit_clicked: Signal<bool> = use_signal(|| false);

    let mut username_error: Signal<Option<String>> = use_signal(|| None);
    let mut email_error: Signal<Option<String>> = use_signal(|| None);
    let mut password_error: Signal<Option<String>> = use_signal(|| None);

    rsx! {
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
                        Err(e) => email_error.set(Some(e.to_string())),
                    }

                    match Password::new(&password.read()) {
                        Ok(_) => password_error.set(None),
                        Err(e) => password_error.set(Some(e.to_string())),
                    }

                    if username_error.read().is_none() && email_error.read().is_none() && password_error.read().is_none() {
                        println!("please make my account");
                    }
                },

                if *submit_clicked.read() {
                    if let Some(error) = username_error.read().as_ref() {
                        div { class : "form-error",
                            "{error}"
                        }
                    }
                }

                div { class : "form-group",
                    label { r#for : "username", ""}
                    input {
                        id : "username",
                        r#type : "text",
                        placeholder : "username",
                        value : "{username}",
                        autocapitalize : "none",
                        spellcheck : "false",
                        oninput : move |event| {
                            username.set(event.value());
                            // Validate in real-time after first submission attempt
                            if *submit_clicked.read() {
                                match Username::new(&event.value()) {
                                    Ok(_) => username_error.set(None),
                                    Err(e) => username_error.set(Some(e.to_string())),
                                }
                            }
                        }
                    }
                }


                if *submit_clicked.read() {
                    if let Some(error) = email_error.read().as_ref() {
                        div { class : "form-error",
                            "{error}"
                        }
                    }
                }

                div { class : "form-group",
                    label { r#for : "email", ""}
                    input {
                        id : "email",
                        r#type : "text",
                        placeholder : "email",
                        value : "{email}",
                        autocapitalize : "none",
                        spellcheck : "false",
                        oninput : move |event| {
                            email.set(event.value());
                            // Validate in real-time after first submission attempt
                            if *submit_clicked.read() {
                                match EmailAddress::from_str(&event.value()) {
                                    Ok(_) => email_error.set(None),
                                    Err(e) => email_error.set(Some(e.to_string())),
                                }
                            }
                        }
                    }
                }

                if *submit_clicked.read() {
                    if let Some(error) = password_error.read().as_ref() {
                        div { class : "form-error",
                            "{error}"
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
                            // Validate in real-time after first submission attempt
                            if *submit_clicked.read() {
                                match Password::new(&event.value()) {
                                    Ok(_) => password_error.set(None),
                                    Err(e) => password_error.set(Some(e.to_string())),
                                }
                            }
                        }
                    }
                }

                button {
                    r#type : "submit",
                    "create"
                }

                button {
                    onclick : move |_| {
                        navigator.push(Route::Login {});
                    },
                    r#type : "submit",
                    class : "login",
                    "back"
                }
            }
        }
    }
}
