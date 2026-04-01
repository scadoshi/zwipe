//! Card format legality filter component.

use dioxus::prelude::*;
use zwipe::domain::{
    card::models::search_card::card_filter::builder::CardFilterBuilder,
    deck::models::deck::format::Format,
};

/// Filter component for format legality (commander, standard, modern, etc.).
#[component]
pub fn FormatFilter() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    rsx! {
        div { class: "flex-col gap-half",
            div { class: "label-row",
                label { class: "label-xs", "format legality" }
                if filter_builder().legalities_contains_any().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_legalities_contains_any();
                        },
                        "\u{00d7}"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 flex-center",
                for fmt in Format::all().iter().copied() {
                    div {
                        class: if filter_builder()
                            .legalities_contains_any()
                            .and_then(|v| v.first())
                            .is_some_and(|k| k == fmt.to_legality_key())
                        {
                            "chip selected"
                        } else {
                            "chip"
                        },
                        onclick: move |_| {
                            let currently_selected = filter_builder()
                                .legalities_contains_any()
                                .and_then(|v| v.first())
                                .is_some_and(|k| k == fmt.to_legality_key());

                            if currently_selected {
                                filter_builder.write().unset_legalities_contains_any();
                            } else {
                                filter_builder.write().set_legalities_contains_any(
                                    vec![fmt.to_legality_key().to_string()]
                                );
                            }
                        },
                        { fmt.display_name().to_lowercase() }
                    }
                }
            }
        }
    }
}
