//! What the app keeps above its screens, as zwiper keeps it above its router: the counters, the color map, the featured card, the oracle tag catalog, and values one screen hands the screen under it.

use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;

use crate::domain::home::{Figures, Scope};

/// How long a fetched set of counters stays fresh.
pub(crate) const COUNTS_FRESH: Duration = Duration::from_secs(60);

/// How long the oracle tag catalog stays fresh before a refetch.
pub(crate) const CATALOG_FRESH: Duration = Duration::from_hours(24);

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

/// The oracle tag catalog as it loads.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) enum Catalog {
    #[default]
    Idle,
    Loading,
    /// The tags and when they arrived. A refetch keeps them on screen.
    Loaded {
        tags: Vec<OracleTag>,
        fetched_at: Instant,
        refreshing: bool,
    },
    Failed,
}

impl Catalog {
    /// The tags, if any have loaded.
    pub(crate) fn tags(&self) -> Option<&[OracleTag]> {
        match self {
            Self::Loaded { tags, .. } => Some(tags),
            _ => None,
        }
    }

    /// The tag's description, `None` while loading or when it has none.
    pub(crate) fn description(&self, slug: &str) -> Option<&str> {
        self.tags()?
            .iter()
            .find(|t| t.slug == slug)?
            .description
            .as_deref()
    }
}

/// Values a screen leaves for the screen under it to take when it comes back into view.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Handoff {
    /// The printing picked on the printing sheet.
    pub(crate) printing: Option<Card>,
    /// The oracle tag the dictionary's Use adopted.
    pub(crate) oracle_tag: Option<String>,
    /// Left by the commander swipe when it closes, so its host refetches and takes the identity its Filter picked.
    pub(crate) commander_swipe: Option<crate::inbound::screens::commander_swipe::Closed>,
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
    pub(crate) oracle_tags: Catalog,
    pub(crate) handoff: Handoff,
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

    /// Whether the catalog should be fetched at `now`: never asked, failed, or loaded more than [`CATALOG_FRESH`] ago and not already refetching.
    pub(crate) fn catalog_due(&self, now: Instant) -> bool {
        match &self.oracle_tags {
            Catalog::Idle | Catalog::Failed => true,
            Catalog::Loading => false,
            Catalog::Loaded {
                fetched_at,
                refreshing,
                ..
            } => !refreshing && now.saturating_duration_since(*fetched_at) >= CATALOG_FRESH,
        }
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

    #[test]
    fn the_catalog_is_due_once_and_again_after_a_day() {
        let now = Instant::now();
        let mut caches = Caches::default();
        assert!(caches.catalog_due(now));
        caches.oracle_tags = Catalog::Loading;
        assert!(!caches.catalog_due(now));
        caches.oracle_tags = Catalog::Loaded {
            tags: Vec::new(),
            fetched_at: now,
            refreshing: false,
        };
        assert!(!caches.catalog_due(now));
        assert!(caches.catalog_due(now + CATALOG_FRESH));
        caches.oracle_tags = Catalog::Loaded {
            tags: Vec::new(),
            fetched_at: now,
            refreshing: true,
        };
        assert!(
            !caches.catalog_due(now + CATALOG_FRESH),
            "one refetch at a time"
        );
        caches.oracle_tags = Catalog::Failed;
        assert!(caches.catalog_due(now));
    }
}
