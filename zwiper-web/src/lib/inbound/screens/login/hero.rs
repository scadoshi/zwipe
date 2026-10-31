//! The mark over everybody's three counters, on the public screens.

use components::{Decode, Panel, StatsStrip};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::logo;
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

/// The app's `HomeHero`: the decoding mark, then decks created, searches run
/// and cards swiped across everyone, rolling until they arrive, tagged
/// `Everybody` and, once fetched, `Live`. A failed fetch is only logged.
#[component]
pub fn HomeHero() -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let mut metrics: Signal<Option<HttpPublicMetrics>> = use_signal(|| None);
    use_future(move || async move {
        let http = client.peek().clone();
        match http.public_metrics().await {
            Ok(fetched) => metrics.set(Some(fetched)),
            Err(e) => tracing::warn!("public counters not fetched: {e}"),
        }
    });
    let figures = public_figures(metrics.read().as_ref());
    let live = metrics.read().is_some();

    rsx! {
        div { class: "home-hero",
            Panel {
                div { class: "hero-head",
                    div { class: "logo", Decode { text: logo::ZWIPE } }
                    StatsStrip {
                        figures,
                        compact: true,
                        source: rsx! {
                            span { class: "stat-chip", "Everybody" }
                            if live {
                                span { class: "stat-chip", "Live" }
                            }
                        },
                    }
                }
            }
        }
    }
}

/// The strip's three figures in the app's order, `None` each while loading.
/// A negative count, which the server never sends, shows as zero.
fn public_figures(metrics: Option<&HttpPublicMetrics>) -> Vec<(Option<u64>, &'static str)> {
    let count = |n: i64| u64::try_from(n).unwrap_or(0);
    vec![
        (metrics.map(|m| count(m.decks_created)), "Decks created"),
        (metrics.map(|m| count(m.searches)), "Searches run"),
        (metrics.map(|m| count(m.cards_swiped)), "Cards swiped"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures_wait_while_loading() {
        assert!(public_figures(None).iter().all(|(n, _)| n.is_none()));
    }

    #[test]
    fn figures_keep_the_app_order() {
        let metrics = HttpPublicMetrics {
            cards_swiped: 3,
            searches: 2,
            decks_created: 1,
        };
        assert_eq!(
            public_figures(Some(&metrics)),
            vec![
                (Some(1), "Decks created"),
                (Some(2), "Searches run"),
                (Some(3), "Cards swiped"),
            ]
        );
    }
}
