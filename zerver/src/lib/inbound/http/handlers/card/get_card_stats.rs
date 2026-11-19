//! One card's public numbers by oracle id.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use zwipe_core::http::contracts::card::HttpCardStats;

use crate::domain::metrics::ports::MetricsService;
use crate::inbound::http::{ApiError, AppState};

/// Returns a card's numbers pooled across every user, read live: its mainboard decks and vested
/// MVPs, its Add-stack swipes, the decks it leads and its Swipe select picks.
///
/// Public, behind the card routes' limiter and hourly cache. A card nobody has touched, or an
/// oracle id the catalog doesn't hold, answers zeros rather than 404.
///
/// # Errors
///
/// Returns 422 for a malformed oracle id and 500 on a database failure.
pub async fn get_card_stats(
    State(state): State<AppState>,
    Path(oracle_id): Path<String>,
) -> Result<(StatusCode, Json<HttpCardStats>), ApiError> {
    let oracle_id = uuid::Uuid::try_parse(&oracle_id)
        .map_err(|e| ApiError::UnprocessableEntity(format!("invalid oracle id: {e}")))?;
    state
        .metrics_service
        .card_stats(oracle_id)
        .await
        .map_err(ApiError::from)
        .map(|stats| (StatusCode::OK, Json(HttpCardStats::from(stats))))
}
