use std::str::FromStr;

use crate::routing::Route;
use dioxus::prelude::*;
use email_address::EmailAddress;
use zwipe::domain::{auth::models::password::Password, user::models::Username};

#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();
    let mut username_or_email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut invalid_credentials = use_signal(|| false);

    rsx! {
        div { class : "form-container",
            h2 { "login" }

            if *invalid_credentials.read() {
                div { class : "form-error",
                    "invalid credentials"
                }
            }

            form {
                onsubmit : move |_| {
                    let valid_identifier = match (
                        Username::new(&username_or_email.read()),
                        EmailAddress::from_str(&username_or_email.read())
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
                },

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

                button {
                    r#type : "submit",
                    "login"
                }

                button {
                    onclick : move |_| {
                        navigator.push(Route::Register {});
                    },
                    r#type : "submit",
                    class : "register",
                    "create profile"
                }
            }
        }
    }
}
