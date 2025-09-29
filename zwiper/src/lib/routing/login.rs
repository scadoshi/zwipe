use crate::routing::Route;
use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    let navigator = use_navigator();
    let mut username_or_email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());
    let mut error = use_signal(|| "".to_string());

    rsx! {
        div { class : "form-container",
            h2 { "login" }

            if !error.read().is_empty() {
                div { class : "form-error",
                    "{error}"
                }
            }

            form {
                onsubmit : move |_| {
                    println!("please log me in");
                },

                div { class : "form-group",
                    label { r#for: "identity", "" }
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
