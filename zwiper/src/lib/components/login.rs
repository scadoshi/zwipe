use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    let mut username_or_email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    rsx! {
        div { class : "login-container",
            h2 { "login" }

            form {
                onsubmit : move |_| {
                    println!("need to build this still");
                },

                div {
                    class : "form-group",
                    label { r#for: "identity", "username or email" }
                    input {
                        id : "identity",
                        r#type : "text",
                        placeholder : "",
                        value : "{username_or_email}",
                        autocapitalize : "none",
                        spellcheck : "false",
                        oninput: move |event| {
                            username_or_email.set(event.value());
                        }
                    }
                }

                div {
                    class : "form-group",
                    label { r#for : "password", "password" }
                    input {
                        id : "password",
                        r#type : "password",
                        placeholder : "",
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
                        println!("need to build this still");
                    },
                    r#type : "submit",
                    class : "register",
                    "create profile"
                }
            }
        }
    }
}
