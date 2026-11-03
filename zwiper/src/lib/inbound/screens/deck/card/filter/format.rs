//! Card format filter component (legality + commander eligibility).

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::format::Format;
use zwipe_ui::filter::chips;
use zwipe_ui::filter::sections::{ChipSet, Modes};

/// Filter component for format legality and commander eligibility.
#[component]
pub fn FormatFilter() -> Element {
    let mut filter_builder: Signal<CardQueryBuilder> = use_context();

    rsx! {
        div { class: "flex-col gap-half",

            // == Commander Eligibility ==
            div { class: "label-row mt-2",
                label { class: "label-xs", "Is commander in" }
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
                        class: if chips::is_selected(
                            ChipSet::CommanderIn,
                            &filter_builder(),
                            fmt.to_legality_key(),
                        ) {
                            "chip selected"
                        } else {
                            "chip"
                        },
                        onclick: move |_| {
                            chips::tap(
                                ChipSet::CommanderIn,
                                &mut filter_builder.write(),
                                &Modes::default(),
                                fmt.to_legality_key(),
                            );
                        },
                        { fmt.display_name().to_string() }
                    }
                }
            }

            // == Format Legality ==
            div { class: "label-row mt-2",
                label { class: "label-xs", "Is legal in" }
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
                        class: if chips::is_selected(
                            ChipSet::LegalIn,
                            &filter_builder(),
                            fmt.to_legality_key(),
                        ) {
                            "chip selected"
                        } else {
                            "chip"
                        },
                        onclick: move |_| {
                            chips::tap(
                                ChipSet::LegalIn,
                                &mut filter_builder.write(),
                                &Modes::default(),
                                fmt.to_legality_key(),
                            );
                        },
                        { fmt.display_name().to_string() }
                    }
                }
            }
        }
    }
}
