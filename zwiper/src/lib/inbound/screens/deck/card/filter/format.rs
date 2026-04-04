//! Card format filter component (legality + commander eligibility).

use dioxus::prelude::*;
use zwipe::domain::{
    card::models::search_card::card_filter::builder::CardFilterBuilder,
    deck::models::deck::format::Format,
};

/// Filter component for format legality and commander eligibility.
#[component]
pub fn FormatFilter() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    rsx! {
        div { class: "flex-col gap-half",

            // ── Commander Eligibility ──────────────────────────────
            div { class: "label-row mt-2",
                label { class: "label-xs", "is commander in" }
                if filter_builder().is_commander_in_format().is_some() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| {
                            filter_builder.write().unset_is_commander_in_format();
                        },
                        "\u{00d7}"
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 flex-center",
                for fmt in Format::commander_formats().iter().copied() {
                    div {
                        class: if filter_builder()
                            .is_commander_in_format()
                            .is_some_and(|f| *f == fmt)
                        {
                            "chip selected"
                        } else {
                            "chip"
                        },
                        onclick: move |_| {
                            let currently_selected = filter_builder()
                                .is_commander_in_format()
                                .is_some_and(|f| *f == fmt);

                            if currently_selected {
                                filter_builder.write().unset_is_commander_in_format();
                            } else {
                                filter_builder.write().set_is_commander_in_format(fmt);
                            }
                        },
                        { fmt.display_name().to_lowercase() }
                    }
                }
            }

            // ── Format Legality ───────────────────────────────────
            div { class: "label-row mt-2",
                label { class: "label-xs", "is legal in" }
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
                            .is_some_and(|v| v.contains(&fmt.to_legality_key().to_string()))
                        {
                            "chip selected"
                        } else {
                            "chip"
                        },
                        onclick: move |_| {
                            let mut current = filter_builder()
                                .legalities_contains_any()
                                .map(|v| v.to_vec())
                                .unwrap_or_default();

                            let key = fmt.to_legality_key().to_string();
                            if current.contains(&key) {
                                current.retain(|k| k != &key);
                            } else {
                                current.push(key);
                            }

                            if current.is_empty() {
                                filter_builder.write().unset_legalities_contains_any();
                            } else {
                                filter_builder.write().set_legalities_contains_any(current);
                            }
                        },
                        { fmt.display_name().to_lowercase() }
                    }
                }
            }
        }
    }
}
