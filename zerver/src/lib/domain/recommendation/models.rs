//! Card-recommendation request + result models for the live deck-aware
//! recommender (the [`CardRecommender`](super::ports::CardRecommender) port).
//!
//! Server-only — the client never calls the recommender — so these live in
//! zerver, not zwipe-core, per the core purity rules.

use uuid::Uuid;

/// A request for deck-aware card recommendations.
///
/// All cards are identified by **oracle_id** (the recommender is queried with
/// `card_format=oracle_id`), so no name resolution is needed on either side.
#[derive(Debug, Clone)]
pub struct RecommendQuery {
    /// The deck's commander.
    pub commander: Uuid,
    /// Optional partner commander.
    pub partner: Option<Uuid>,
    /// Current deck contents (oracle_ids), excluding the commander.
    pub deck: Vec<Uuid>,
}

/// One recommended card.
#[derive(Debug, Clone)]
pub struct CardRecommendation {
    /// The recommended card's oracle_id.
    pub oracle_id: Uuid,
    /// Human-readable card name (for logging/diagnostics).
    pub name: String,
    /// Recommendation score; higher is stronger.
    pub score: f64,
}

/// Failure modes of a recommendation call.
///
/// Every variant is non-fatal to the caller: the read path falls back to the
/// cached synergy signal on any error, so a deck-card search never fails
/// because the recommender did.
#[derive(Debug, thiserror::Error)]
pub enum RecommendError {
    /// The recommender is disabled by configuration (kill switch).
    #[error("recommender disabled")]
    Disabled,
    /// Network, timeout, or I/O failure calling the recommender.
    #[error("network error: {0}")]
    Network(#[source] anyhow::Error),
    /// The recommender returned a non-success status or result code
    /// (e.g. `error_rate_limited`, `error_booting`, `error_model_loading`).
    #[error("recommender api error (status {status}, code {code}): {}", messages.join("; "))]
    Api {
        /// HTTP status code.
        status: u16,
        /// Upstream `result_code` (or `http_<status>` for a non-JSON body).
        code: String,
        /// Any human-readable messages the API returned.
        messages: Vec<String>,
    },
    /// The response body could not be parsed.
    #[error("failed to parse recommender response: {0}")]
    Parse(#[source] anyhow::Error),
}
