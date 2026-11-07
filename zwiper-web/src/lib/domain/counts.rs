//! Everybody's counters: one shared copy above the router, fetched at most
//! once a minute.

use chrono::{DateTime, Duration, Utc};
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;
use zwipe_ui::home::CountsTag;
use zwipe_ui::timing::COUNTS_FRESH;

/// The counters as last known, held above the router so every hero shares
/// them.
#[derive(Debug, Clone)]
pub struct CachedCounts {
    /// What the server answered.
    pub metrics: HttpPublicMetrics,
    /// When it answered during this visit; `None` for the copy stored by an
    /// earlier visit, which shows until a fetch replaces it.
    pub fetched_at: Option<DateTime<Utc>>,
    /// When the server's copy was made, which an edge cache can put well
    /// before `fetched_at`; `None` when it did not say.
    pub made_at: Option<DateTime<Utc>>,
}

impl CachedCounts {
    /// The copy an earlier visit stored.
    pub fn restored(metrics: HttpPublicMetrics) -> Self {
        Self {
            metrics,
            fetched_at: None,
            made_at: None,
        }
    }

    /// A copy fetched at `at`, made by the server at `made_at`.
    pub fn fetched(
        metrics: HttpPublicMetrics,
        at: DateTime<Utc>,
        made_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            metrics,
            fetched_at: Some(at),
            made_at,
        }
    }

    /// Restored, or older than zwipe-ui's [`COUNTS_FRESH`] at `now`, so Home
    /// fetches again. The copy stays on screen while it does.
    pub fn is_stale(&self, now: DateTime<Utc>) -> bool {
        let fresh = Duration::from_std(COUNTS_FRESH).unwrap_or(Duration::MAX);
        self.fetched_at
            .is_none_or(|at| now.signed_duration_since(at) >= fresh)
    }

    /// Fetched during this visit, so the hero tags it.
    pub fn is_live(&self) -> bool {
        self.fetched_at.is_some()
    }

    /// The hero's tag at `now`: `Live` while the server's copy is new, "as
    /// of" its time after; `None` for a copy from an earlier visit.
    pub fn tag(&self, now: DateTime<Utc>) -> Option<CountsTag> {
        self.is_live().then(|| CountsTag::at(self.made_at, now))
    }
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
    fn a_restored_copy_is_stale_and_not_live() {
        let cached = CachedCounts::restored(metrics());
        assert!(cached.is_stale(Utc::now()));
        assert!(!cached.is_live());
    }

    #[test]
    fn a_fresh_copy_serves_for_a_minute() {
        let at = Utc::now();
        let cached = CachedCounts::fetched(metrics(), at, None);
        assert!(cached.is_live());
        assert!(!cached.is_stale(at + Duration::seconds(59)));
        assert!(cached.is_stale(at + Duration::seconds(60)));
    }
}
