//! A password field with a Show/Hide toggle, for any form that takes a
//! password. Styled by `assets/components.css`.

use dioxus::prelude::*;

/// A password input whose Show/Hide toggle reveals what is typed. `class`
/// appends the host's own input classes; the host's label and error line sit
/// outside it.
#[component]
pub fn PasswordInput(
    value: Signal<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] placeholder: Option<String>,
    #[props(default = false)] disabled: bool,
    #[props(default)] class: Option<String>,
) -> Element {
    let mut shown = use_signal(|| false);
    let input_class = match class {
        Some(class) => format!("{class} input-password"),
        None => "input-password".to_string(),
    };

    rsx! {
        div { class: "password-input-wrapper",
            input {
                class: "{input_class}",
                id,
                r#type: if shown() { "text" } else { "password" },
                placeholder,
                value: "{value}",
                disabled,
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| value.set(event.value()),
            }
            button {
                r#type: "button",
                class: "password-toggle-btn",
                aria_label: if shown() { "Hide password" } else { "Show password" },
                onclick: move |_| shown.toggle(),
                if shown() { "Hide" } else { "Show" }
            }
        }
    }
}
