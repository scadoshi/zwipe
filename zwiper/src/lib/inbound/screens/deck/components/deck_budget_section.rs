//! Budget rows for the deck view's collapsible "Budget" section: land target,
//! actual land count, price target, and the deck's running total / average card
//! price. Split out of the profile card (mirrors the Tags section) so the
//! profile stays compact. The USD/EUR/TIX chips live in the section header and
//! drive the running-price rows; the price *target* keeps its own stored
//! currency (Scryfall gives each card a native price per currency, so switching
//! reads a different pre-summed field, never a conversion). Rendered inside a
//! [`CollapsibleSection`](super::collapsible_section::CollapsibleSection).

use dioxus::prelude::*;
use zwipe_core::domain::{
    card::search_card::card_filter::price_currency::PriceCurrency,
    deck::{deck_metrics::DeckMetrics, deck_profile::DeckProfile},
};

/// Whether the deck has any budget-relevant field to show: a target set, or
/// live metrics (which carry the land count + running prices).
pub(crate) fn has_budget(p: &DeckProfile, has_metrics: bool) -> bool {
    p.land_target.is_some() || p.price_target.is_some() || has_metrics
}

/// Budget rows: land target, land count, price target, running total + average
/// price. Land target and Lands sit adjacent (goal vs. actual).
#[component]
pub(crate) fn DeckBudgetSection(
    deck_profile: DeckProfile,
    metrics: Option<DeckMetrics>,
    /// Currency selected via the header chips; drives the running-price rows.
    selected_currency: Signal<&'static str>,
) -> Element {
    let currency = PriceCurrency::from_key(selected_currency()).unwrap_or_default();
    let (total, avg) = match (&metrics, currency) {
        (Some(m), PriceCurrency::Eur) => (m.total_price_eur, m.avg_price_eur),
        (Some(m), PriceCurrency::Tix) => (m.total_price_tix, m.avg_price_tix),
        (Some(m), PriceCurrency::Usd) => (m.total_price_usd, m.avg_price_usd),
        (None, _) => (None, None),
    };
    let fmt = |val: Option<f64>| match val {
        Some(v) => currency.format_amount(v),
        None => "N/A".to_string(),
    };

    rsx! {
        div { style: "display:flex;flex-direction:column;",
            // Lands: actual count, shown as `actual / target` when a land target is set.
            if metrics.is_some() || deck_profile.land_target.is_some() {
                {
                    let count = metrics.as_ref().map(|m| m.land_count).unwrap_or(0);
                    let value = match deck_profile.land_target {
                        Some(target) => format!("{count} / {target}"),
                        None => count.to_string(),
                    };
                    rsx! {
                        div { class: "info-row",
                            span { class: "info-row-label", "Lands" }
                            span { class: "info-row-value", "{value}" }
                        }
                    }
                }
            }
            // Total price: running total, shown as `total / target` when a price
            // target is set. The target keeps its own stored currency.
            if metrics.is_some() || deck_profile.price_target.is_some() {
                {
                    // A running total is a sum, so an empty/priceless deck reads as
                    // the currency's zero, not "N/A".
                    let total_fmt = currency.format_amount(total.unwrap_or(0.0));
                    let value = match deck_profile.price_target {
                        Some(target) => {
                            let target_currency = deck_profile.price_target_currency.unwrap_or_default();
                            format!("{total_fmt} / {}", target_currency.format_amount(target))
                        }
                        None => total_fmt,
                    };
                    rsx! {
                        div { class: "info-row",
                            span { class: "info-row-label", "Total price" }
                            span { class: "info-row-value", "{value}" }
                        }
                    }
                }
            }
            if metrics.is_some() {
                div { class: "info-row",
                    span { class: "info-row-label", "Average card price" }
                    span { class: "info-row-value", "{fmt(avg)}" }
                }
            }
        }
    }
}
