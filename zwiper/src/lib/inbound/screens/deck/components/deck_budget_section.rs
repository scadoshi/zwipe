//! Budget rows for the deck view's collapsible "Budget" section: land target and
//! price target. Split out of the profile card (mirrors the Tags section) so the
//! profile stays compact. Rendered inside a
//! [`CollapsibleSection`](super::collapsible_section::CollapsibleSection).

use dioxus::prelude::*;
use zwipe_core::domain::deck::deck_profile::DeckProfile;

/// Whether the deck has any budget field set (worth showing the section for).
pub(crate) fn has_budget(p: &DeckProfile) -> bool {
    p.land_target.is_some() || p.price_target.is_some()
}

/// The budget rows (land target, price target), each shown only when set.
#[component]
pub(crate) fn DeckBudgetSection(deck_profile: DeckProfile) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;",
            if let Some(target) = deck_profile.land_target {
                div { class: "info-row",
                    span { class: "info-row-label", "Land target" }
                    span { class: "info-row-value", "{target}" }
                }
            }
            if let Some(budget) = deck_profile.price_target {
                {
                    let currency = deck_profile.price_target_currency.unwrap_or_default();
                    rsx! {
                        div { class: "info-row",
                            span { class: "info-row-label", "Price target" }
                            span { class: "info-row-value", "{currency.format_amount(budget)}" }
                        }
                    }
                }
            }
        }
    }
}
