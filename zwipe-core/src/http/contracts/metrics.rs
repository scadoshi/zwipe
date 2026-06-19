//! User metrics HTTP request/response contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Batched usage counters posted by the client.
///
/// The client buffers counts in memory and flushes periodically (every ~30s
/// and on screen-exit / app backgrounding). All fields are additive — the
/// server increments the corresponding lifetime and daily counters.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HttpUsageBatch {
    /// Right swipes (add card to deck).
    pub swipes_right: u32,
    /// Left swipes (skip card).
    pub swipes_left: u32,
    /// Up swipes.
    pub swipes_up: u32,
    /// Down swipes.
    pub swipes_down: u32,
    /// Card searches issued.
    pub searches: u32,
}

impl HttpUsageBatch {
    /// Maximum accepted value per counter per flush.
    ///
    /// The client buffers only ~30s of activity before flushing, so legitimate
    /// values are tiny (tens). This caps untrusted input so a client can't
    /// inflate the lifetime / public-marketing totals, and keeps each day's
    /// accumulation comfortably within the daily-activity `INTEGER` columns
    /// even at the endpoint's request rate limit.
    pub const MAX_PER_FLUSH: u32 = 10_000;

    /// Returns a copy with every counter clamped to [`Self::MAX_PER_FLUSH`].
    #[must_use]
    pub fn clamped(&self) -> Self {
        Self {
            swipes_right: self.swipes_right.min(Self::MAX_PER_FLUSH),
            swipes_left: self.swipes_left.min(Self::MAX_PER_FLUSH),
            swipes_up: self.swipes_up.min(Self::MAX_PER_FLUSH),
            swipes_down: self.swipes_down.min(Self::MAX_PER_FLUSH),
            searches: self.searches.min(Self::MAX_PER_FLUSH),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HttpUsageBatch;

    #[test]
    fn clamped_caps_each_field_and_leaves_small_ones() {
        let clamped = HttpUsageBatch {
            swipes_right: u32::MAX,
            swipes_left: 5,
            swipes_up: HttpUsageBatch::MAX_PER_FLUSH + 1,
            swipes_down: 0,
            searches: u32::MAX,
        }
        .clamped();
        assert_eq!(clamped.swipes_right, HttpUsageBatch::MAX_PER_FLUSH);
        assert_eq!(clamped.swipes_left, 5);
        assert_eq!(clamped.swipes_up, HttpUsageBatch::MAX_PER_FLUSH);
        assert_eq!(clamped.swipes_down, 0);
        assert_eq!(clamped.searches, HttpUsageBatch::MAX_PER_FLUSH);
    }
}

/// Public app-wide aggregates surfaced on zwipe.net.
///
/// Counts span all users. Numbers are sums over `user_lifetime_counters` at
/// query time; the endpoint is cached at the CF edge for ~1h to keep origin
/// load near zero.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpPublicMetrics {
    /// Sum of every swipe across every user (right + left + up + down).
    pub cards_swiped: i64,
    /// Sum of every card search across every user.
    pub searches: i64,
    /// Decks created across every user.
    pub decks_created: i64,
}

/// Per-user lifetime metric totals returned by `GET /api/user/metrics`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpLifetimeCounters {
    /// Owning user id.
    pub user_id: Uuid,
    /// Total right swipes.
    pub swipes_right: i64,
    /// Total left swipes.
    pub swipes_left: i64,
    /// Total up swipes.
    pub swipes_up: i64,
    /// Total down swipes.
    pub swipes_down: i64,
    /// Total searches.
    pub searches: i64,
    /// Decks created.
    pub decks_created: i32,
    /// Decks that have reached a valid state at least once.
    pub decks_completed: i32,
    /// Last write to this counter row. Not a last-active signal —
    /// `users.last_active_at` is the canonical one.
    pub updated_at: DateTime<Utc>,
}
