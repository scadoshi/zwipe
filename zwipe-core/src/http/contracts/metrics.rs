//! User metrics HTTP request/response contracts.

use chrono::NaiveDateTime;
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
    /// Last write to this row — doubles as last-active timestamp.
    pub updated_at: NaiveDateTime,
}
