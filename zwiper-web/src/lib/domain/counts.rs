//! Everybody's counters on the public screens: one shared copy, fetched at
//! most once a minute.

use chrono::{DateTime, Duration, Utc};
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

/// How long a fetched copy serves before the next screen with the hero asks
/// again, so moving between Login, Register and Forgot password reuses it.
pub const COUNTS_FRESH: Duration = Duration::seconds(60);

/// The counters as last known, held above the router so every hero shares
/// them.
#[derive(Debug, Clone)]
pub struct CachedCounts {
    /// What the server answered.
    pub metrics: HttpPublicMetrics,
    /// When it answered during this visit; `None` for the copy stored by an
    /// earlier visit, which shows until a fetch replaces it.
    pub fetched_at: Option<DateTime<Utc>>,
}

impl CachedCounts {
    /// The copy an earlier visit stored.
    pub fn restored(metrics: HttpPublicMetrics) -> Self {
        Self {
            metrics,
            fetched_at: None,
        }
    }

    /// A copy fetched at `at`.
    pub fn fetched(metrics: HttpPublicMetrics, at: DateTime<Utc>) -> Self {
        Self {
            metrics,
            fetched_at: Some(at),
        }
    }

    /// Restored, or older than [`COUNTS_FRESH`] at `now`, so a mounting hero
    /// fetches again. The copy stays on screen while it does.
    pub fn is_stale(&self, now: DateTime<Utc>) -> bool {
        self.fetched_at
            .is_none_or(|at| now.signed_duration_since(at) >= COUNTS_FRESH)
    }

    /// Fetched during this visit, which the hero tags `Live`.
    pub fn is_live(&self) -> bool {
        self.fetched_at.is_some()
    }
}

/// Whether a hero mounting at `now` should fetch: no copy, a restored one,
/// or one past [`COUNTS_FRESH`].
pub fn needs_fetch(cache: Option<&CachedCounts>, now: DateTime<Utc>) -> bool {
    cache.is_none_or(|cached| cached.is_stale(now))
}

/// The strip's three figures in the app's order, `None` each while loading.
/// A negative count, which the server never sends, shows as zero.
pub fn public_figures(metrics: Option<&HttpPublicMetrics>) -> Vec<(Option<u64>, &'static str)> {
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

    fn metrics() -> HttpPublicMetrics {
        HttpPublicMetrics {
            cards_swiped: 3,
            searches: 2,
            decks_created: 1,
        }
    }

    #[test]
    fn nothing_cached_fetches() {
        assert!(needs_fetch(None, Utc::now()));
    }

    #[test]
    fn a_restored_copy_fetches_and_is_not_live() {
        let cached = CachedCounts::restored(metrics());
        assert!(needs_fetch(Some(&cached), Utc::now()));
        assert!(!cached.is_live());
    }

    #[test]
    fn a_fresh_copy_serves_for_a_minute() {
        let at = Utc::now();
        let cached = CachedCounts::fetched(metrics(), at);
        assert!(cached.is_live());
        assert!(!needs_fetch(Some(&cached), at + Duration::seconds(59)));
        assert!(needs_fetch(Some(&cached), at + Duration::seconds(60)));
    }

    #[test]
    fn figures_wait_while_loading() {
        assert!(public_figures(None).iter().all(|(n, _)| n.is_none()));
    }

    #[test]
    fn figures_keep_the_app_order() {
        assert_eq!(
            public_figures(Some(&metrics())),
            vec![
                (Some(1), "Decks created"),
                (Some(2), "Searches run"),
                (Some(3), "Cards swiped"),
            ]
        );
    }

    #[test]
    fn a_negative_count_shows_zero() {
        let mut odd = metrics();
        odd.searches = -4;
        assert_eq!(public_figures(Some(&odd))[1], (Some(0), "Searches run"));
    }
}
