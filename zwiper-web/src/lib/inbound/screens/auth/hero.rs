//! The mark over three counters: the panel Home's scoped hero draws in, and
//! everybody's counts it shows.

use chrono::Utc;
use components::{Decode, Panel, StatsStrip};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::logo;
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;
use zwipe_ui::home::Figures;

use crate::domain::counts::{self, CachedCounts};
use crate::outbound::counts_store::CountsStore;

/// The strip's three figures with their labels, `None` each while loading.
pub fn strip_figures(figures: Option<Figures>) -> Vec<(Option<u64>, &'static str)> {
    let values = figures.unwrap_or_default().values();
    values.into_iter().zip(Figures::LABELS).collect()
}

/// Everybody's counters from the root's shared copy, with whether that copy
/// was fetched during this visit. A mount refetches when the copy is
/// missing, stored by an earlier visit, or a minute old, and keeps showing
/// it until the fetch lands. A failed fetch is only logged.
pub fn use_public_counts() -> (Option<HttpPublicMetrics>, bool) {
    let client: Signal<ZwipeClient> = use_context();
    let mut cache: Signal<Option<CachedCounts>> = use_context();
    use_hook(move || {
        if !counts::needs_fetch(cache.peek().as_ref(), Utc::now()) {
            return;
        }
        spawn(async move {
            let http = client.peek().clone();
            match http.public_metrics().await {
                Ok(metrics) => {
                    CountsStore::browser().save(&metrics);
                    cache.set(Some(CachedCounts::fetched(metrics, Utc::now())));
                }
                Err(e) => tracing::warn!("public counters not refreshed: {e}"),
            }
        });
    });
    let cached = cache.read();
    let metrics = cached.as_ref().map(|c| c.metrics.clone());
    let live = cached.as_ref().is_some_and(CachedCounts::is_live);
    (metrics, live)
}

/// The panel both heroes draw: the mark over the strip, with `source`
/// under it. A new `roll_key` mounts the strip afresh, so its figures roll
/// to the new counts.
#[component]
pub fn HeroPanel(
    figures: Vec<(Option<u64>, &'static str)>,
    roll_key: &'static str,
    source: Element,
) -> Element {
    rsx! {
        div { class: "home-hero",
            Panel {
                div { class: "hero-head",
                    div { class: "logo", Decode { text: logo::ZWIPE } }
                    for roll_key in [roll_key] {
                        StatsStrip {
                            key: "{roll_key}",
                            figures: figures.clone(),
                            compact: true,
                            source: source.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures_wait_while_loading_and_keep_the_app_order() {
        assert!(strip_figures(None).iter().all(|(n, _)| n.is_none()));
        let metrics = HttpPublicMetrics {
            cards_swiped: 3,
            searches: 2,
            decks_created: 1,
        };
        assert_eq!(
            strip_figures(Some(Figures::from(&metrics))),
            vec![
                (Some(1), "Decks created"),
                (Some(2), "Searches run"),
                (Some(3), "Cards swiped"),
            ]
        );
    }
}
