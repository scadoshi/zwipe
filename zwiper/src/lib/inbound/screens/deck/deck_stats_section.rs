use dioxus::prelude::*;
use zwipe::domain::deck::models::deck_metrics::DeckMetrics;

#[component]
pub(super) fn DeckStatsSection(metrics: DeckMetrics) -> Element {
    rsx! {
        label { class: "label", "stats" }
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
        }
    }
}
