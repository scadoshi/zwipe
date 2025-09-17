use dioxus::prelude::*;

#[component]
pub fn Login() -> Element {
    // These are our reactive state variables
    let mut username_or_email = use_signal(|| "".to_string());
    let mut password = use_signal(|| "".to_string());

    // Handler for form submission
    let handle_submit = move |event: Event<FormData>| {
        event.prevent_default(); // Prevent page refresh
        println!("login attempt:");
        println!("  identity: {}", username_or_email.read());
        println!("  password: {}", password.read());
        // TODO: Call your backend API here
    };

    rsx! {
        div { class: "login-container",
            h2 { "login" }

            form {
                onsubmit: handle_submit,

                div { class: "form-group",
                    label { r#for: "identity", "username or email" }
                    input {
                        id: "identity",
                        r#type: "text",
                        placeholder: "",
                        value: "{username_or_email}",
                        autocapitalize: "none",
                        spellcheck: "false",
                        oninput: move |event| {
                            username_or_email.set(event.value());
                        }
                    }
                }

                div { class: "form-group",
                    label { r#for: "password", "password" }
                    input {
                        id: "password",
                        r#type: "password",
                        placeholder: "",
                        value: "{password}",
                        autocapitalize: "none",
                        spellcheck: "false",
                        oninput: move |event| {
                            password.set(event.value());
                        }
                    }
                }

                button {
                    r#type: "submit",
                    "login"
                }
            }
        }
    }
}
