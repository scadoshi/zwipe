use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::{
    OrderByOptions, builder::CardFilterBuilder,
};

#[component]
pub fn Sort() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    rsx! {
        div { class: "flex-col gap-half",
            // Order By selection
            div { class: "label-row",
                label { class: "label-xs", "sort by" }
                if filter_builder().order_by().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_order_by();
                        },
                        "×"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 flex-center",
                for option in OrderByOptions::all() {
                    div {
                        class: if filter_builder().order_by() == Some(option) {
                            "chip selected"
                        } else {
                            "chip"
                        },
                        onclick: move |_| {
                            let current = filter_builder().order_by();
                            if current == Some(option) {
                                filter_builder.write().unset_order_by();
                            } else {
                                filter_builder.write().set_order_by(option);
                            }
                        },
                        { option.to_string().to_lowercase() }
                    }
                }
            }

            div { class: "label-row",
                label { class: "label-xs", "sort order" }
            }

            // Ascending/Descending toggle
            div { class: "flex flex-wrap gap-1 flex-center",
                div {
                    class: "chip",
                    onclick: move |_| {
                        let current = filter_builder().ascending();
                        filter_builder.write().set_ascending(!current);
                    },
                    { if filter_builder().ascending() { "ascending" } else { "descending" } }
                }
            }
        }
    }
}
