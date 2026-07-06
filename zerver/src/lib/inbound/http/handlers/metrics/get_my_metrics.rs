use axum::{Json, extract::State, http::StatusCode};

use crate::inbound::http::{ApiError, AppState, middleware::AuthenticatedUser};
use zwipe_core::http::contracts::metrics::HttpLifetimeCounters;

/// Returns the caller's lifetime metric totals.
pub async fn get_my_metrics(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<HttpLifetimeCounters>), ApiError> {
    let counters = state
        .metrics_service
        .lifetime_counters(user.id)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(HttpLifetimeCounters::from(counters))))
}
