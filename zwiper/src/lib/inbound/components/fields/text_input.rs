use dioxus::prelude::*;

#[component]
pub fn TextInput(
    value: Signal<String>,
    id: Option<String>,
    label: Option<String>,
    placeholder: Option<String>,
    input_type: Option<String>,
) -> Element {
    let id = id.unwrap_or_default();
    let placeholder = placeholder.unwrap_or_default();
    let input_type = input_type.unwrap_or_else(|| "text".to_string());
    let is_password = input_type == "password";

    let mut show_password = use_signal(|| false);
    let effective_type = if is_password && show_password() {
        "text".to_string()
    } else {
        input_type
    };

    rsx! {
        if let Some(label) = label {
            label { class: "label", r#for : "{id}", "{label}" }
        }

        if is_password {
            div { class: "password-input-wrapper",
                input { class: "input input-password",
                    id : "{id}",
                    r#type : "{effective_type}",
                    placeholder : "{placeholder}",
                    value : "{value}",
                    autocapitalize : "none",
                    autocorrect : "off",
                    spellcheck : "false",
                    oninput: move |event| {
                        value.set(event.value());
                    }
                }
                button {
                    r#type: "button",
                    class: "password-toggle-btn",
                    "aria-label": if show_password() { "Hide password" } else { "Show password" },
                    onclick: move |_| show_password.set(!show_password()),
                    if show_password() { "Hide" } else { "Show" }
                }
            }
        } else {
            input { class: "input",
                id : "{id}",
                r#type : "{effective_type}",
                placeholder : "{placeholder}",
                value : "{value}",
                autocapitalize : "none",
                autocorrect : "off",
                spellcheck : "false",
                oninput: move |event| {
                    value.set(event.value());
                }
            }
        }
    }
}
