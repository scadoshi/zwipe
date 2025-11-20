use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::SearchCards;

#[component]
pub fn Text(filter: Signal<SearchCards>) -> Element {
    rsx! {
        label { class: "label", r#for : "name-contains", "name contains" }
        input { class : "input",
            id : "name-contains",
            placeholder : "name contains",
            value : if let Some(name) = filter.read().name_contains.as_deref() {
                name
            } else { "" },
            r#type : "text",
            autocapitalize : "none",
            spellcheck : "false",
            oninput : move |event| {
                filter.write().name_contains = Some(event.value());
                if filter.read().name_contains == Some("".to_string()) {
                    filter.write().name_contains = None;
                }
            }
        }
    }
}
