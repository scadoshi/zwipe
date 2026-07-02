//! Card name filter component.

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;

/// Filter component for card name search.
#[component]
pub fn Name() -> Element {
    let mut filter_builder: Signal<CardQueryBuilder> = use_context();

    let name_value = filter_builder().name_contains().unwrap_or("").to_string();

    let name_not_value = filter_builder()
        .name_not_contains()
        .unwrap_or("")
        .to_string();

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "name-contains", "Name contains" }
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
                placeholder: "Name contains",
                value: name_value,
                r#type: "text",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| {
                    filter_builder.write().set_name_contains(event.value());
                }
            }
            div { class: "label-row mt-2",
                label { class: "label-xs", r#for: "name-not-contains", "Name doesn't contain" }
                if filter_builder().name_not_contains().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_name_not_contains();
                        },
                        "×"
                    }
                }
            }
            input { class: "input input-compact",
                id: "name-not-contains",
                placeholder: "Name doesn't contain",
                value: name_not_value,
                r#type: "text",
                autocapitalize: "none",
                autocorrect: "off",
                spellcheck: "false",
                oninput: move |event| {
                    filter_builder.write().set_name_not_contains(event.value());
                }
            }
        }
    }
}
