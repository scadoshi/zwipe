//! Flavor text filter component.

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;

/// Filter component for card flavor text search.
#[component]
pub fn FlavorText() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let flavor_text_value = filter_builder()
        .flavor_text_contains()
        .unwrap_or("")
        .to_string();

    let flavor_text_not_value = filter_builder()
        .flavor_text_not_contains()
        .unwrap_or("")
        .to_string();

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "flavor-text-contains", "Flavor text contains" }
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
                placeholder: "Flavor text contains",
                value: flavor_text_value,
                r#type: "text",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| {
                    filter_builder.write().set_flavor_text_contains(event.value());
                }
            }
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "flavor-text-not-contains", "Flavor text doesn't contain" }
                if filter_builder().flavor_text_not_contains().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_flavor_text_not_contains();
                        },
                        "×"
                    }
                }
            }
            input { class: "input input-compact",
                id: "flavor-text-not-contains",
                placeholder: "Flavor text doesn't contain",
                value: flavor_text_not_value,
                r#type: "text",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| {
                    filter_builder.write().set_flavor_text_not_contains(event.value());
                }
            }
        }
    }
}
