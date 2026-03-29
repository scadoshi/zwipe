//! Filter configuration accordion component.

use crate::inbound::components::tri_toggle::TriToggle;
use dioxus::prelude::*;
use zwipe::domain::card::models::search_card::card_filter::builder::CardFilterBuilder;

/// Configuration panel with language, reprint, and promo filters.
#[component]
pub fn Config() -> Element {
    let filter_builder: Signal<CardFilterBuilder> = use_context();

    rsx! {
        div { class: "flex-col gap-1",
            TriToggle {
                label: "is playable",
                filter_builder,
                getter: |fb| fb.is_playable(),
                setter_true: |fb| { fb.set_is_playable(true); },
                setter_false: |fb| { fb.set_is_playable(false); },
                unsetter: |fb| { fb.unset_is_playable(); },
                true_label: "yes",
                false_label: "no",
                none_label: "any"
            }

            TriToggle {
                label: "is digital",
                filter_builder,
                getter: |fb| fb.digital(),
                setter_true: |fb| { fb.set_digital(true); },
                setter_false: |fb| { fb.set_digital(false); },
                unsetter: |fb| { fb.unset_digital(); },
                true_label: "yes",
                false_label: "no",
                none_label: "any"
            }

            TriToggle {
                label: "is oversized",
                filter_builder,
                getter: |fb| fb.oversized(),
                setter_true: |fb| { fb.set_oversized(true); },
                setter_false: |fb| { fb.set_oversized(false); },
                unsetter: |fb| { fb.unset_oversized(); },
                true_label: "yes",
                false_label: "no",
                none_label: "any"
            }

            TriToggle {
                label: "is promo",
                filter_builder,
                getter: |fb| fb.promo(),
                setter_true: |fb| { fb.set_promo(true); },
                setter_false: |fb| { fb.set_promo(false); },
                unsetter: |fb| { fb.unset_promo(); },
                true_label: "yes",
                false_label: "no",
                none_label: "any"
            }

            TriToggle {
                label: "has content warning",
                filter_builder,
                getter: |fb| fb.content_warning(),
                setter_true: |fb| { fb.set_content_warning(true); },
                setter_false: |fb| { fb.set_content_warning(false); },
                unsetter: |fb| { fb.unset_content_warning(); },
                true_label: "yes",
                false_label: "no",
                none_label: "any"
            }
        }
    }
}
