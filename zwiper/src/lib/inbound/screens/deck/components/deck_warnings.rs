//! Deck warnings section with action buttons for card-specific warnings, and the Unpriced entry
//! whose cards open on the Cards.

use components::{Button, ButtonVariant, Chip};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_warning::DeckWarning;
use zwipe_ui::deck::warning::{self, Fix};
use zwipe_ui::screens::deck_view::{self, Action};

/// The fix buttons' warning color.
const FIX_STYLE: &str = "color: var(--ink-color-warning, var(--color-warning)); border-color: var(--border-warning); margin-bottom: 0;";

#[component]
pub(crate) fn DeckWarnings(
    warnings: Vec<DeckWarning>,
    /// The cards with no price in `currency`, the Unpriced entry's; it shows while there is one.
    unpriced: Vec<Card>,
    currency: PriceCurrency,
    /// Opens an unpriced card's details by its printing ID.
    on_open_unpriced: EventHandler<Uuid>,
    ondispatch: EventHandler<Action>,
) -> Element {
    let none_left = warnings.is_empty() && unpriced.is_empty();
    rsx! {
        div { style: "display:flex;flex-direction:column;",
            if !unpriced.is_empty() {
                UnpricedRow { cards: unpriced, currency, on_open: on_open_unpriced }
            }
            // Once a fix clears the last warning the panel stays and says so.
            if none_left {
                div { class: "settings-row deck-warning-row",
                    span { class: "text-muted", "{deck_view::NO_WARNINGS_LEFT}" }
                }
            }
            for warning in warnings.iter() {
                div { class: "settings-row deck-warning-row",
                    span { class: "text-muted", "{warning}" }
                    // The below-minimum warning's two fixes sit together;
                    // every other warning has one.
                    if warning::fix(warning) == Some(Fix::BelowMinimum) {
                        span { class: "deck-warning-fixes",
                            for (label, action) in deck_view::fix_buttons(warning) {
                                Button {
                                    variant: ButtonVariant::Small,
                                    style: FIX_STYLE,
                                    onclick: move |_| ondispatch.call(action.clone()),
                                    "{label}"
                                }
                            }
                        }
                    } else {
                        for (label, action) in deck_view::fix_buttons(warning) {
                            Button {
                                variant: ButtonVariant::Small,
                                style: FIX_STYLE,
                                onclick: move |_| ondispatch.call(action.clone()),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The Unpriced entry: how many cards have no price, and on the right a Show chip that, selected
/// and reading Hide, lays the cards out as chips under it in its row, as a role's tag chips reveal
/// theirs. It starts unselected.
#[component]
fn UnpricedRow(cards: Vec<Card>, currency: PriceCurrency, on_open: EventHandler<Uuid>) -> Element {
    let mut shown = use_signal(|| false);
    let line = warning::unpriced_line(cards.len(), currency);
    rsx! {
        // One row: the chips wrap onto their own line under the count, inside it.
        div { class: "settings-row deck-warning-row",
            span { class: "text-muted", "{line}" }
            Chip { selected: shown(), onclick: move |_| shown.toggle(),
                if shown() { {warning::HIDE_UNPRICED} } else { {warning::SHOW_UNPRICED} }
            }
            // The tag chips' reveal, so the cards ease open and shut; shut, they take no focus.
            div {
                class: if shown() { "keyword-reveal open deck-unpriced-fold" } else { "keyword-reveal deck-unpriced-fold" },
                inert: (!shown()).then_some("true"),
                div { class: "keyword-reveal-inner",
                    div { class: "chip-row deck-unpriced-chips",
                        for card in cards {
                            Chip {
                                key: "{card.scryfall_data.id}",
                                selected: false,
                                onclick: move |_| on_open.call(card.scryfall_data.id),
                                "{card.scryfall_data.name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
