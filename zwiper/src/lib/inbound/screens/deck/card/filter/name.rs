//! Card name filter component.

use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

/// Filter component for card name search.
#[component]
pub fn Name() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let name_value = filter_builder().name_contains().unwrap_or("").to_string();

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row",
                label { class: "label-xs", r#for: "name-contains", "name contains" }
                if filter_builder().name_contains().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_name_contains();
                        },
                        "×"
                    }
                }
            }
            input { class: "input input-compact",
                id: "name-contains",
                placeholder: "name contains",
                value: name_value,
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    filter_builder.write().set_name_contains(event.value());
                }
            }
        }
    }
}
