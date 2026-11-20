//! Counts one card search run signed out on zwipe.net (no auth, no body).

use axum::extract::State;
use axum::http::StatusCode;

use crate::domain::metrics::ports::MetricsService;
use crate::inbound::http::{ApiError, AppState};

/// Adds one card search to the public searches total, for a search with no account behind it.
///
/// It reads nothing from the request, so a body counts nothing more and nothing about the search
/// is kept; the route's IP rate limit bounds how fast one address can raise the total.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn record_anonymous_search(
    State(state): State<AppState>,
) -> Result<StatusCode, ApiError> {
    state
        .metrics_service
        .record_anonymous_search()
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
