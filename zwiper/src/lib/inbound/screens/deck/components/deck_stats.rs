use dioxus::prelude::*;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;

#[component]
pub(crate) fn DeckStats(metrics: DeckMetrics, show_buy_sheet: Signal<bool>) -> Element {
    let mut selected_currency = use_signal(|| "usd");

    let (total, avg, symbol) = match selected_currency() {
        "eur" => (metrics.total_price_eur, metrics.avg_price_eur, "€"),
        "tix" => (metrics.total_price_tix, metrics.avg_price_tix, ""),
        _ => (metrics.total_price_usd, metrics.avg_price_usd, "$"),
    };

    let fmt = |val: Option<f64>| match val {
        Some(v) => format!("{symbol}{v:.2}"),
        None => "n/a".to_string(),
    };

    rsx! {
        div { style: "display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 0.4rem;",
            label { class: "label", style: "margin-bottom: 0;", "stats" }
            div { class: "chip-row", style: "margin-bottom: 0; align-items: center;",
                for (label, key) in [("usd", "usd"), ("eur", "eur"), ("tix", "tix")] {
                    div {
                        class: if selected_currency() == key { "chip selected" } else { "chip" },
                        onclick: move |_| selected_currency.set(key),
                        "{label}"
                    }
                }
                span { class: "text-muted", "|" }
                div {
                    class: "chip",
                    onclick: move |_| show_buy_sheet.set(true),
                    "buy"
                }
            }
        }
        div { class: "info-list",
            div { class: "info-row",
                span { class: "info-row-label", "cards" }
                span { class: "info-row-value", "{metrics.total_cards}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "avg cmc" }
                span { class: "info-row-value", "{metrics.avg_cmc:.1}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "lands" }
                span { class: "info-row-value", "{metrics.land_count}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "total price" }
                span { class: "info-row-value", "{fmt(total)}" }
            }
            div { class: "info-row",
                span { class: "info-row-label", "avg card price" }
                span { class: "info-row-value", "{fmt(avg)}" }
            }
        }

    }
}
