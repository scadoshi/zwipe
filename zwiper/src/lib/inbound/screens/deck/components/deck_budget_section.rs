//! Budget rows for the deck view's collapsible "Budget" section: the deck's
//! running total, against the price target when it has one, and the average
//! card price. The USD/EUR/TIX chips in the section header pick the currency
//! the deck view's model reads the rows in; the target keeps its own stored
//! currency. Rendered inside a
//! [`CollapsibleSection`](super::collapsible_section::CollapsibleSection).

use dioxus::prelude::*;
use zwipe_ui::screens::deck_view;

/// Budget rows: the running total and the average card price, each shown
/// when the model has a value for it.
#[component]
pub(crate) fn DeckBudgetSection(total: Option<String>, average: Option<String>) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;",
            if let Some(value) = total {
                div { class: "settings-row",
                    span { class: "settings-row-label", {deck_view::TOTAL_PRICE} }
                    span { class: "info-row-value", "{value}" }
                }
            }
            if let Some(value) = average {
                div { class: "settings-row",
                    span { class: "settings-row-label", {deck_view::AVERAGE_CARD_PRICE} }
                    span { class: "info-row-value", "{value}" }
                }
            }
        }
    }
}
