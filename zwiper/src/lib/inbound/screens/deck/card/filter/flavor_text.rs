//! Flavor text filter component.

use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

/// Filter component for card flavor text search.
#[component]
pub fn FlavorText() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let flavor_text_value = filter_builder()
        .flavor_text_contains()
        .unwrap_or("")
        .to_string();

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row",
                label { class: "label-xs", r#for: "flavor-text-contains", "flavor text contains" }
                if filter_builder().flavor_text_contains().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_flavor_text_contains();
                        },
                        "×"
                    }
                }
            }
            input { class: "input input-compact",
                id: "flavor-text-contains",
                placeholder: "flavor text contains",
                value: flavor_text_value,
                r#type: "text",
                autocapitalize: "none",
                spellcheck: "false",
                oninput: move |event| {
                    filter_builder.write().set_flavor_text_contains(event.value());
                }
            }
        }
    }
}
