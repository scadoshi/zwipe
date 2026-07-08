//! Live aggregate stats strip surfaced on the marketing site.
//!
//! Fetches `GET {API_BASE}/api/marketing/stats` during SSR. CF caches the
//! API response at the edge (~2h TTL), GH Pages caches the rendered HTML,
//! so cost-per-pageview is near zero. On error the strip hides itself;
//! don't break the marketing page on a metrics outage.

use crate::API_BASE;
use dioxus::prelude::*;
use zwipe_core::http::{contracts::metrics::HttpPublicMetrics, paths::public_metrics_route};

#[component]
pub fn StatsStrip() -> Element {
    let stats: Resource<Option<HttpPublicMetrics>> = use_resource(|| async {
        let url = format!("{}{}", API_BASE, public_metrics_route());
        let res = reqwest::Client::new().get(&url).send().await.ok()?;
        if !res.status().is_success() {
            return None;
        }
        res.json::<HttpPublicMetrics>().await.ok()
    });

    let value = stats.read();
    let Some(Some(s)) = &*value else {
        return rsx! {};
    };

    rsx! {
        hr { class: "hero-rule" }
        section { class: "stats-strip",
            div { class: "stat",
                span { class: "stat-num", "{format_count(s.cards_swiped)}" }
                span { class: "stat-label", "Cards swiped" }
            }
            div { class: "stat",
                span { class: "stat-num", "{format_count(s.searches)}" }
                span { class: "stat-label", "Searches run" }
            }
            div { class: "stat",
                span { class: "stat-num", "{format_count(s.decks_created)}" }
                span { class: "stat-label", "Decks created" }
            }
        }
    }
}

/// 12345 -> "12,345"
fn format_count(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 {
        out.push('-');
    }
    out.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::format_count;

    #[test]
    fn formats_small_numbers() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(42), "42");
        assert_eq!(format_count(999), "999");
    }

    #[test]
    fn formats_thousands() {
        assert_eq!(format_count(1_000), "1,000");
        assert_eq!(format_count(12_345), "12,345");
        assert_eq!(format_count(123_456), "123,456");
        assert_eq!(format_count(1_234_567), "1,234,567");
    }
}
