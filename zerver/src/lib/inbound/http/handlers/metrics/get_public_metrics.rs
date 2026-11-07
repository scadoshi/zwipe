//! Returns public app-wide aggregate metrics (no auth).

use axum::Json;
use axum::extract::State;
use axum::http::{StatusCode, header};
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

use crate::domain::metrics::ports::MetricsService;
use crate::inbound::http::{ApiError, AppState};

/// The counters' `Cache-Control`. Each CF edge location holds a copy for
/// five minutes and serves it a minute longer while it fetches the next, and
/// a browser holds one for a minute, so every copy in front of the origin is
/// within a few minutes of it. The clients tag an older copy with its age.
pub const PUBLIC_METRICS_CACHE: &str =
    "public, max-age=60, s-maxage=300, stale-while-revalidate=60";

/// Returns app-wide aggregate metrics (cards swiped, searches, decks created).
///
/// Public: no auth required. Sent with [`PUBLIC_METRICS_CACHE`], which the
/// router's hourly default leaves in place.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn get_public_metrics(
    State(state): State<AppState>,
) -> Result<
    (
        StatusCode,
        [(header::HeaderName, &'static str); 1],
        Json<HttpPublicMetrics>,
    ),
    ApiError,
> {
    let metrics = state
        .metrics_service
        .public_metrics()
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::OK,
        [(header::CACHE_CONTROL, PUBLIC_METRICS_CACHE)],
        Json(HttpPublicMetrics::from(metrics)),
    ))
}
