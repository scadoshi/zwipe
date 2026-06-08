//! App-wide aggregate metrics surfaced to the marketing site.

use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

/// Sums across every user's lifetime counters. Source of truth for the
/// numbers shown on zwipe.net.
#[derive(Debug, Clone)]
pub struct PublicMetrics {
    /// Sum of every swipe (right + left + up + down) across every user.
    pub cards_swiped: i64,
    /// Sum of every card search across every user.
    pub searches: i64,
    /// Decks created across every user.
    pub decks_created: i64,
}

impl From<PublicMetrics> for HttpPublicMetrics {
    fn from(value: PublicMetrics) -> Self {
        Self {
            cards_swiped: value.cards_swiped,
            searches: value.searches,
            decks_created: value.decks_created,
        }
    }
}
