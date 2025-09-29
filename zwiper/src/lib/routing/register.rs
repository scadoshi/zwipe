use dioxus::prelude::*;

use crate::routing::Route;

#[component]
pub fn Register() -> Element {
    let navigator = use_navigator();
    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    rsx! {
        div { class : "form-container",
            h2 { "create profile" }

            form {
                onsubmit : move |_| {
                    println!("please create profile");
                },

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
