//! Quick-add search bar for the deck cards screen.
//!
//! Draws the deck cards model's Quick add: the typed query, and the results
//! floating over the content above. The search itself (debounce, the
//! deck-aware request, the add) is `zwipe_ui::screens::deck_cards`'.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_ui::screens::deck_cards::{self, QuickAddResults};

/// Grows the float anchor by exactly the height the results overflow past the
/// scrollport top (back toward 0 when they fit with slack), so results overlay
/// without reflow in most cases and spend real space only when they'd run
/// under the screen header. Self-correcting: each run adds the current
/// signed deficit to the anchor's current height, so it converges as results
/// change. Runs on the next frame (post-layout) and once more after the
/// focus smooth-scroll has settled.
const QUICK_ADD_FIT_JS: &str = r"
const fit = () => {
    const anchor = document.getElementById('quick-add-anchor');
    const results = document.getElementById('quick-add-results');
    const content = document.querySelector('.screen-content');
    if (!anchor || !results || !content) return;
    // Rendered height (not the inline target) so a mid-transition remeasure
    // still lands on the right total instead of compounding. The 8px gap
    // keeps the top chip row from sitting flush against the header's border.
    const gap = 8;
    const current = anchor.getBoundingClientRect().height;
    const deficit = content.getBoundingClientRect().top + gap - results.getBoundingClientRect().top;
    anchor.style.height = Math.max(0, current + deficit) + 'px';
};
requestAnimationFrame(fit);
setTimeout(fit, 400);
";

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
    // Measured overlay: results float over the content above (no reflow) in
    // the common case; when they'd run past the scrollport top, under the
    // screen header; the fit effect below grows the anchor by exactly the
    // overflow, spending real space only in those edge cases.
    use_effect(use_reactive!(|results| {
        if results.is_some() {
            let _ = document::eval(QUICK_ADD_FIT_JS);
        }
    }));

    rsx! {
        div {
            // Results float in a layer hung above the bar from the (measured,
            // normally zero-height) anchor. Tap a chip to add it to the
            // mainboard.
            if let Some(results) = results {
                div { class: "search-float-anchor", id: "quick-add-anchor",
                    match results {
                        QuickAddResults::Searching => rsx! {
                            div { class: "search-float-results", id: "quick-add-results",
                                div { class: "chip-unselected search-float-chip", "{deck_cards::SEARCHING}" }
                            }
                        },
                        QuickAddResults::Empty => rsx! {
                            div { class: "search-float-results", id: "quick-add-results",
                                div { class: "chip-unselected search-float-chip", "{deck_cards::NO_RESULTS}" }
                            }
                        },
                        QuickAddResults::Found(cards) => rsx! {
                            div { class: "search-float-results", id: "quick-add-results",
                                for (i, card) in cards.iter().cloned().enumerate() {
                                    div {
                                        class: "chip-unselected search-float-chip",
                                        style: "animation-delay: {i * 40}ms, {250 + i * 40}ms;",
                                        onclick: move |_| onadd(card.clone()),
                                        "{card.scryfall_data.name}"
                                    }
                                }
                            }
                        },
                    }
                }
            }

            // Label sits inline-left like the Group by / Boards / Show rows,
            // in the same accent-tertiary `chip-row-label`; the input fills the rest.
            div { class: "chip-row",
                span { class: "stat-chip stat-chip-p1 chip-row-label", "{deck_cards::QUICK_ADD}" }
                input {
                    class: "input input-compact",
                    style: "flex: 1; margin-bottom: 0;",
                    id: "quick-add",
                    r#type: "text",
                    placeholder: deck_cards::QUICK_ADD_PLACEHOLDER,
                    value: "{query}",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    // Scroll back to the top on focus so the header above the
                    // bar (the float's canvas) is in view before results
                    // appear, instead of them tucking under the screen header.
                    onfocus: move |_| {
                        let _ = document::eval(
                            "const el = document.querySelector('.screen-content'); if (el) el.scrollTo({ top: 0, behavior: 'smooth' });",
                        );
                    },
                    oninput: move |event| oninput(event.value()),
                }
                if !query.is_empty() {
                    button {
                        class: "clear-btn",
                        onclick: move |_| onclear(()),
                        "\u{00d7}"
                    }
                }
            }
        }
    }
}
