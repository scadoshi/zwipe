//! Public client-metadata handlers (app version gating).

use axum::{Json, extract::State, http::StatusCode};

use zwipe_core::http::contracts::client::HttpMinClientVersion;

use crate::inbound::http::AppState;

/// Returns the minimum app version this server supports.
///
/// Public and unauthenticated: stale clients must be able to learn they're
/// gated without a valid session. The value comes from `MIN_CLIENT_VERSION`
/// in the server env; `"0.0.0"` means the gate is open.
pub async fn get_min_client_version(
    State(state): State<AppState>,
) -> (StatusCode, Json<HttpMinClientVersion>) {
    (
        StatusCode::OK,
        Json(HttpMinClientVersion {
            min_version: state.min_client_version.to_string(),
        }),
    )
}
