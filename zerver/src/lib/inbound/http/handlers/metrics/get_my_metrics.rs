//! Returns the caller's lifetime counters.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::http::contracts::metrics::HttpLifetimeCounters;

use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState};

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
