use crate::inbound::components::{hint_host::HintTopic, info_button::InfoButton};
use dioxus::prelude::*;
use zwipe_components::PasswordInput;

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
    let input_class = if is_error { "input input-error" } else { "input" };

    rsx! {
        if let Some(label) = label {
            if let Some(topic) = hint {
                // The label alone is centered, like the labels without a "?";
                // the "?" hangs off its right edge.
                div { class: "label-hint",
                    span { class: "label-hint-anchor",
                        label { class: "label", r#for : "{id}", "{label}" }
                        InfoButton { topic }
                    }
                }
            } else {
                label { class: "label", r#for : "{id}", "{label}" }
            }
        }

        if is_password {
            PasswordInput {
                value,
                id,
                placeholder,
                class: input_class.to_string(),
            }
        } else {
            input { class: input_class,
                id : "{id}",
                r#type : "{input_type}",
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
