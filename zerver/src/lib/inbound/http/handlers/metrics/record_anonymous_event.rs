use axum::{Json, extract::State, http::StatusCode};

use crate::inbound::http::{ApiError, AppState};
use zwipe_core::http::contracts::metrics::HttpAnonymousEvent;

/// Records one pre-auth funnel event (no auth — there is no user yet).
///
/// The kind is a closed enum in the contract, so an unknown kind is rejected
/// at deserialization; the IP rate limit on the route bounds row volume.
pub async fn record_anonymous_event(
    State(state): State<AppState>,
    Json(event): Json<HttpAnonymousEvent>,
) -> Result<StatusCode, ApiError> {
    state
        .metrics_service
        .record_anonymous_event(event.session_id, event.kind)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
