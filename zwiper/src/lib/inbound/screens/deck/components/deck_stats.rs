use dioxus::prelude::*;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;

#[component]
pub(crate) fn DeckStats(
    metrics: DeckMetrics,
    /// Currency selected via the header chips (shared with the Stats header).
    selected_currency: Signal<&'static str>,
) -> Element {
    let (total, avg, symbol) = match selected_currency() {
        "eur" => (metrics.total_price_eur, metrics.avg_price_eur, "€"),
        "tix" => (metrics.total_price_tix, metrics.avg_price_tix, ""),
        _ => (metrics.total_price_usd, metrics.avg_price_usd, "$"),
    };

    let fmt = |val: Option<f64>| match val {
        Some(v) => format!("{symbol}{v:.2}"),
        None => "N/A".to_string(),
    };

    rsx! {
        div { style: "display:flex;flex-direction:column;",
            div { class: "info-row",
                span { class: "info-row-label", "Cards" }
                span { class: "info-row-value", "{metrics.total_cards}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "Average mana value" }
                span { class: "info-row-value", "{metrics.avg_cmc:.1}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "Lands" }
                span { class: "info-row-value", "{metrics.land_count}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "Total price" }
                span { class: "info-row-value", "{fmt(total)}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "Average card price" }
                span { class: "info-row-value", "{fmt(avg)}" }
            }
        }
    }
}
