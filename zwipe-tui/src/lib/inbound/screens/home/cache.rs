//! What the app keeps above its screens, as zwiper keeps it above its router: the counters, the color map, the featured card, the oracle tag catalog, and values one screen hands the screen under it.

use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;

use crate::domain::home::{Figures, Scope};

/// How long a fetched set of counters stays fresh.
pub(crate) const COUNTS_FRESH: Duration = Duration::from_secs(60);

/// Everybody's counters and when they were fetched; `None` for the copy restored from the last launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CachedCounts {
    pub(crate) figures: Figures,
    pub(crate) fetched_at: Option<Instant>,
}

/// One user's own counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CachedMyCounts {
    pub(crate) user_id: Uuid,
    pub(crate) figures: Figures,
    pub(crate) fetched_at: Instant,
}

/// One user's color map summary.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CachedPlayStyle {
    pub(crate) user_id: Uuid,
    pub(crate) summary: HttpPlayStyleSummary,
}

/// The featured card and the top of the UTC hour it stops being featured.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FlavorCard {
    pub(crate) card: Card,
    pub(crate) expires_at: DateTime<Utc>,
}

/// Everything kept above the screens.
#[derive(Debug, Default)]
pub(crate) struct Caches {
    pub(crate) counts: Option<CachedCounts>,
    pub(crate) my_counts: Option<CachedMyCounts>,
    pub(crate) play_style: Option<CachedPlayStyle>,
    pub(crate) flavor: Option<FlavorCard>,
    /// Whether this launch has said hello.
    pub(crate) greeted: bool,
    pub(crate) scope: Scope,
    /// Whether the counters and scope saved by the last launch have been asked for.
    pub(crate) restore_asked: bool,
}

impl Caches {
    /// Whether everybody's counters need fetching at `now`: missing, restored from the last launch, or older than [`COUNTS_FRESH`].
    pub(crate) fn counts_stale(&self, now: Instant) -> bool {
        self.counts
            .and_then(|c| c.fetched_at)
            .is_none_or(|at| now.saturating_duration_since(at) >= COUNTS_FRESH)
    }

    /// Whether `user`'s own counters need fetching at `now`: missing, another user's, or older than [`COUNTS_FRESH`].
    pub(crate) fn my_counts_stale(&self, user: Uuid, now: Instant) -> bool {
        self.my_counts.is_none_or(|c| {
            c.user_id != user || now.saturating_duration_since(c.fetched_at) >= COUNTS_FRESH
        })
    }

    /// `user`'s own counters, never another user's.
    pub(crate) fn my_figures(&self, user: Option<Uuid>) -> Option<Figures> {
        self.my_counts
            .filter(|c| Some(c.user_id) == user)
            .map(|c| c.figures)
    }

    /// `user`'s color map summary, never another user's.
    pub(crate) fn play_style_of(&self, user: Option<Uuid>) -> Option<&HttpPlayStyleSummary> {
        self.play_style
            .as_ref()
            .filter(|c| Some(c.user_id) == user)
            .map(|c| &c.summary)
    }

    /// The featured card while it is still this hour's at `now`.
    pub(crate) fn flavor_expired(&self, now: DateTime<Utc>) -> bool {
        self.flavor.as_ref().is_none_or(|f| now >= f.expires_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_are_stale_when_restored_or_a_minute_old() {
        let now = Instant::now();
        let mut caches = Caches::default();
        assert!(caches.counts_stale(now));
        caches.counts = Some(CachedCounts {
            figures: Figures::default(),
            fetched_at: None,
        });
        assert!(caches.counts_stale(now), "a restored copy refetches");
        caches.counts = Some(CachedCounts {
            figures: Figures::default(),
            fetched_at: Some(now),
        });
        assert!(!caches.counts_stale(now + Duration::from_secs(59)));
        assert!(caches.counts_stale(now + COUNTS_FRESH));
    }

    #[test]
    fn my_counts_belong_to_one_user() {
        let now = Instant::now();
        let me = Uuid::new_v4();
        let caches = Caches {
            my_counts: Some(CachedMyCounts {
                user_id: me,
                figures: Figures::default(),
                fetched_at: now,
            }),
            ..Caches::default()
        };
        assert!(!caches.my_counts_stale(me, now));
        assert!(caches.my_counts_stale(Uuid::new_v4(), now));
        assert!(caches.my_figures(Some(Uuid::new_v4())).is_none());
        assert!(caches.my_figures(Some(me)).is_some());
    }
}
