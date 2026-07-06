use axum::{Json, extract::State, http::StatusCode};

use crate::inbound::http::{ApiError, AppState};
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

/// Returns app-wide aggregate metrics (cards swiped, searches, decks created).
///
/// Public — no auth required. Cached at the CF edge for ~1h so the origin
/// only sees one hit per POP per TTL window.
pub async fn get_public_metrics(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<HttpPublicMetrics>), ApiError> {
    let metrics = state
        .metrics_service
        .public_metrics()
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(HttpPublicMetrics::from(metrics))))
}
