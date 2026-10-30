//! Stores a crash report posted on the launch after a crash (no auth).

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::http::contracts::metrics::HttpCrashReport;

use crate::inbound::http::{ApiError, AppState};

/// Stores one crash report (no auth: the next launch after a crash may have
/// no session, and a crash in the auth flow must still be reportable).
///
/// The unauthed-write defense stack lives around, not in, this handler: a
/// strict per-IP governor and a small body cap on the route, the repo's
/// server-side clamp, and `ON CONFLICT (crash_id) DO NOTHING` making replays
/// free. Always 204 on success: the client deletes its crash file on any 2xx.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn record_crash(
    State(state): State<AppState>,
    Json(report): Json<HttpCrashReport>,
) -> Result<StatusCode, ApiError> {
    state
        .metrics_service
        .record_crash(&report)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
