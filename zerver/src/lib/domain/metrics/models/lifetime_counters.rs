//! Per-user lifetime counter aggregate.

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::http::contracts::metrics::HttpLifetimeCounters;

/// Lifetime totals for one user, fetched as a single row.
#[derive(Debug, Clone)]
pub struct LifetimeCounters {
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
    /// Total searches issued.
    pub searches: i64,
    /// Decks created lifetime.
    pub decks_created: i32,
    /// Decks that have reached a valid state at least once.
    pub decks_completed: i32,
    /// Last write to this counter row. Not a last-active signal;
    /// `users.last_active_at` is the canonical one.
    pub updated_at: DateTime<Utc>,
}

impl LifetimeCounters {
    /// The counters of an account that has counted nothing: every total zero, as of `at`.
    /// What an account without a counters row reads as.
    #[must_use]
    pub fn zero(user_id: Uuid, at: DateTime<Utc>) -> Self {
        Self {
            user_id,
            swipes_right: 0,
            swipes_left: 0,
            swipes_up: 0,
            swipes_down: 0,
            searches: 0,
            decks_created: 0,
            decks_completed: 0,
            updated_at: at,
        }
    }
}

impl From<LifetimeCounters> for HttpLifetimeCounters {
    fn from(value: LifetimeCounters) -> Self {
        Self {
            user_id: value.user_id,
            swipes_right: value.swipes_right,
            swipes_left: value.swipes_left,
            swipes_up: value.swipes_up,
            swipes_down: value.swipes_down,
            searches: value.searches,
            decks_created: value.decks_created,
            decks_completed: value.decks_completed,
            updated_at: value.updated_at,
        }
    }
}
