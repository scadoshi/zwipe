//! Free-text oracle text search component.

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;

/// Free-text oracle text contains input.
#[component]
pub(crate) fn TextContains() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let oracle_text_value = filter_builder()
        .oracle_text_contains()
        .unwrap_or("")
        .to_string();

    let oracle_text_not_value = filter_builder()
        .oracle_text_not_contains()
        .unwrap_or("")
        .to_string();

    rsx! {
        div { class: "label-row mt-2",
            label { class: "label-xs", r#for: "oracle-text-contains", "Oracle text contains" }
            if filter_builder().oracle_text_contains().is_some() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        filter_builder.write().unset_oracle_text_contains();
                    },
                    "\u{00d7}"
                }
            }
        }
        input { class: "input input-compact",
            id: "oracle-text-contains",
            placeholder: "Oracle text contains",
            value: oracle_text_value,
            r#type: "text",
            autocapitalize: "none",
            spellcheck: "false",
            oninput: move |event| {
                filter_builder.write().set_oracle_text_contains(event.value());
            }
        }
        div { class: "label-row mt-2",
            label { class: "label-xs", r#for: "oracle-text-not-contains", "Oracle text doesn't contain" }
            if filter_builder().oracle_text_not_contains().is_some() {
                button {
                    class: "clear-btn",
                    onclick: move |_| {
                        filter_builder.write().unset_oracle_text_not_contains();
                    },
                    "\u{00d7}"
                }
            }
        }
        input { class: "input input-compact",
            id: "oracle-text-not-contains",
            placeholder: "Oracle text doesn't contain",
            value: oracle_text_not_value,
            r#type: "text",
            autocapitalize: "none",
            spellcheck: "false",
            oninput: move |event| {
                filter_builder.write().set_oracle_text_not_contains(event.value());
            }
        }
    }
}
