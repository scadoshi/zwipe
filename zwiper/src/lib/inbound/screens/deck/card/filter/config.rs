use crate::inbound::components::tri_toggle::TriToggle;
use dioxus::prelude::*;
use zwipe::domain::card::models::{
    scryfall_data::language::Language, search_card::card_filter::builder::CardFilterBuilder,
};

#[component]
pub fn Config() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    rsx! {
        div { class: "flex-col gap-1",
            // Language chips section
            div { class: "flex-col gap-half",
                label { class: "label-xs", "language" }
                div { class: "flex flex-wrap gap-1",
                    for lang in Language::all() {
                        div {
                            class: if filter_builder().language().is_some_and(|l| l == lang) {
                                "chip selected"
                            } else {
                                "chip"
                            },
                            onclick: move |_| {
                                if filter_builder().language().is_some_and(|l| l == lang) {
                                    filter_builder.write().unset_language();
                                } else {
                                    filter_builder.write().set_language(lang);
                                }
                            },
                            { lang.to_name().to_lowercase() }
                        }
                    }
                }
            }

            // Tri-state filters section
            TriToggle {
                label: "playable",
                filter_builder,
                getter: |fb| fb.is_playable(),
                setter_true: |fb| { fb.set_is_playable(true); },
                setter_false: |fb| { fb.set_is_playable(false); },
                unsetter: |fb| { fb.unset_is_playable(); },
                true_label: "show",
                false_label: "hide",
                none_label: "neither"
            }

            TriToggle {
                label: "digital",
                filter_builder,
                getter: |fb| fb.digital(),
                setter_true: |fb| { fb.set_digital(true); },
                setter_false: |fb| { fb.set_digital(false); },
                unsetter: |fb| { fb.unset_digital(); },
                true_label: "show",
                false_label: "hide",
                none_label: "neither"
            }

            TriToggle {
                label: "oversized",
                filter_builder,
                getter: |fb| fb.oversized(),
                setter_true: |fb| { fb.set_oversized(true); },
                setter_false: |fb| { fb.set_oversized(false); },
                unsetter: |fb| { fb.unset_oversized(); },
                true_label: "show",
                false_label: "hide",
                none_label: "neither"
            }

            TriToggle {
                label: "promo",
                filter_builder,
                getter: |fb| fb.promo(),
                setter_true: |fb| { fb.set_promo(true); },
                setter_false: |fb| { fb.set_promo(false); },
                unsetter: |fb| { fb.unset_promo(); },
                true_label: "show",
                false_label: "hide",
                none_label: "neither"
            }

            TriToggle {
                label: "content warning",
                filter_builder,
                getter: |fb| fb.content_warning(),
                setter_true: |fb| { fb.set_content_warning(true); },
                setter_false: |fb| { fb.set_content_warning(false); },
                unsetter: |fb| { fb.unset_content_warning(); },
                true_label: "show",
                false_label: "hide",
                none_label: "neither"
            }
        }
    }
}
