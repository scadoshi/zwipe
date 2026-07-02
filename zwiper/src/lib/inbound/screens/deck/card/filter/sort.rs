//! Sort order selection component.

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;

/// Component for selecting card sort order and direction.
#[component]
pub fn Sort() -> Element {
    let mut filter_builder: Signal<CardQueryBuilder> = use_context();

    rsx! {
        div { class: "flex-col gap-half",
            // Order By selection
            div { class: "label-row mt-2",
                label { class: "label-xs", "Sort by" }
                if filter_builder().sort().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_sort();
                        },
                        "×"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 flex-center",
                for option in CardSortKey::all() {
                    div {
                        class: if filter_builder().sort() == Some(option) {
                            "chip selected"
                        } else {
                            "chip"
                        },
                        onclick: move |_| {
                            let current = filter_builder().sort();
                            if current == Some(option) {
                                filter_builder.write().unset_sort();
                            } else {
                                filter_builder.write().set_sort(option);
                            }
                        },
                        { option.to_string() }
                    }
                }
            }

            div { class: "label-row mt-2",
                label { class: "label-xs", "Sort order" }
            }

            // Ascending/Descending toggle
            div { class: "flex flex-wrap gap-1 flex-center",
                div {
                    class: "chip",
                    onclick: move |_| {
                        let current = filter_builder().ascending();
                        filter_builder.write().set_ascending(!current);
                    },
                    { if filter_builder().ascending() { "Ascending" } else { "Descending" } }
                }
            }
        }
    }
}
