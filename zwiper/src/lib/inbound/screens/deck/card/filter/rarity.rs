use dioxus::prelude::*;
use zwipe::domain::card::models::{
    scryfall_data::rarity::Rarity as CardRarity,
    search_card::card_filter::builder::CardFilterBuilder,
};

#[component]
pub fn Rarity() -> Element {
    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    rsx! {
        div { class: "flex-col gap-half",
            label { class: "label-xs", "rarity" }

            div { class: "flex flex-wrap gap-1 flex-center",
                for rarity in CardRarity::all() {
                    div {
                        class: if filter_builder()
                            .rarity_equals_any()
                            .is_some_and(|r| r.contains(&rarity))
                        {
                            "type-box-compact selected"
                        } else {
                            "type-box-compact"
                        },
                        onclick: move |_| {
                            let current = filter_builder()
                                .rarity_equals_any()
                                .map(|r| r.to_vec())
                                .unwrap_or_default();

                            let mut new = current;
                            if new.contains(&rarity) {
                                new.retain(|r| r != &rarity);
                            } else {
                                new.push(rarity);
                            }

                            if new.is_empty() {
                                filter_builder.write().unset_rarity_equals_any();
                            } else {
                                filter_builder.write().set_rarity_equals_any(new.into());
                            }
                        },
                        { rarity.to_long_name() }
                    }
                }
            }
        }
    }
}
