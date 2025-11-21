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
    let input_type = input_type.unwrap_or("text".to_string());

    rsx! {
        if let Some(label) = label {
            label { class: "label", r#for : "{id}", "{label}" }
        }

        input { class: "input",
            id : "{id}",
            r#type : "{input_type}",
            placeholder : "{placeholder}",
            value : "{value}",
            autocapitalize : "none",
            spellcheck : "false",
            oninput: move |event| {
                value.set(event.value());
            }
        }
    }
}
