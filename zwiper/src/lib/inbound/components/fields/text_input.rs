use crate::inbound::components::{hint_host::HintTopic, info_button::InfoButton};
use dioxus::prelude::*;

/// Labeled text input. `hint` puts a "?" right of the label that opens that
/// topic's explainer.
#[component]
pub fn TextInput(
    value: Signal<String>,
    id: Option<String>,
    label: Option<String>,
    placeholder: Option<String>,
    input_type: Option<String>,
    error: Option<String>,
    hint: Option<HintTopic>,
) -> Element {
    let id = id.unwrap_or_default();
    let placeholder = placeholder.unwrap_or_default();
    let input_type = input_type.unwrap_or_else(|| "text".to_string());
    let is_password = input_type == "password";
    let is_error = error.is_some();

    let mut show_password = use_signal(|| false);
    let effective_type = if is_password && show_password() {
        "text".to_string()
    } else {
        input_type
    };

    rsx! {
        if let Some(label) = label {
            if let Some(topic) = hint {
                // The label keeps its own spacing below; this row only pairs
                // it with the "?", which stands a little taller than the text.
                // Centered, because the form centers its labels and a flex
                // row would otherwise pull this one to the left edge.
                div { style: "display:flex;align-items:center;justify-content:center;",
                    label { class: "label", r#for : "{id}", "{label}" }
                    InfoButton { topic }
                }
            } else {
                label { class: "label", r#for : "{id}", "{label}" }
            }
        }

        if is_password {
            div { class: "password-input-wrapper",
                input { class: if is_error { "input input-password input-error" } else { "input input-password" },
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
            input { class: if is_error { "input input-error" } else { "input" },
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

        if let Some(error) = error {
            div { class: "message-error", "{error}" }
        }
    }
}
