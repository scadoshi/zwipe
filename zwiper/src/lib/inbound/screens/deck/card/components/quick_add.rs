//! Quick-add search bar for the deck cards screen.
//!
//! Draws the deck cards model's Quick add: the typed query, and the results
//! in flow under it, pushing the list down. The search itself (debounce, the
//! deck-aware request, the add, which closes the results) is
//! `zwipe_ui::screens::deck_cards`'.

use components::ClearWrap;
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_ui::screens::deck_cards::{self, QuickAddResults};

/// Live card-name search that adds the picked card straight to the mainboard.
///
/// `results` is `None` while the dropdown is closed.
#[component]
pub fn QuickAdd(
    query: String,
    results: Option<QuickAddResults>,
    oninput: EventHandler<String>,
    onclear: EventHandler,
    onadd: EventHandler<Card>,
) -> Element {
    rsx! {
        div {
            // Label sits inline-left like the Group by / Boards / Show rows,
            // in the same accent-tertiary `chip-row-label`; the input fills the rest.
            div { class: "chip-row",
                label { class: "stat-chip stat-chip-p1 chip-row-label", r#for: "quick-add", "{deck_cards::QUICK_ADD}" }
                ClearWrap {
                    filled: !query.is_empty(),
                    on_clear: move |()| onclear(()),
                    label: "Clear the search".to_string(),
                    input {
                        class: "input input-compact",
                        id: "quick-add",
                        r#type: "text",
                        placeholder: deck_cards::QUICK_ADD_PLACEHOLDER,
                        value: "{query}",
                        autocapitalize: "none",
                        autocorrect: "off",
                        spellcheck: "false",
                        // Scroll back to the top on focus so the bar and the
                        // results that open under it are in view.
                        onfocus: move |_| {
                            let _ = document::eval(
                                "const el = document.querySelector('.screen-content'); if (el) el.scrollTo({ top: 0, behavior: 'smooth' });",
                            );
                        },
                        oninput: move |event| oninput(event.value()),
                    }
                }
            }

            // The results under the bar. Tap a chip to add it to the
            // mainboard.
            if let Some(results) = results {
                div {
                    match results {
                        QuickAddResults::Searching => rsx! {
                            div { class: "card-filter-results quick-add-results",
                                div { class: "chip-unselected card-filter-result", "{deck_cards::SEARCHING}" }
                            }
                        },
                        QuickAddResults::Empty => rsx! {
                            div { class: "card-filter-results quick-add-results",
                                div { class: "chip-unselected card-filter-result", "{deck_cards::NO_RESULTS}" }
                            }
                        },
                        QuickAddResults::Found(cards) => rsx! {
                            div { class: "card-filter-results quick-add-results",
                                for (i, card) in cards.iter().cloned().enumerate() {
                                    button {
                                        r#type: "button",
                                        class: "chip-unselected card-filter-result",
                                        style: "animation-delay: {i * 40}ms;",
                                        onclick: move |_| onadd(card.clone()),
                                        "{card.scryfall_data.name}"
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}
