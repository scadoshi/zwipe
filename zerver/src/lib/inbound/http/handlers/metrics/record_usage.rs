//! Accepts a batched usage update from the client.

use std::time::Instant;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::http::contracts::metrics::HttpUsageBatch;

use crate::domain::metrics::models::errors::MetricsError;
use crate::domain::metrics::models::kinds::EventKind;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<MetricsError> for ApiError {
    fn from(value: MetricsError) -> Self {
        match value {
            MetricsError::NotFound => Self::NotFound("metrics row not found".to_string()),
            MetricsError::Database(e) => e.to_500(),
        }
    }
}

/// Increments lifetime and daily counters for the caller by the batch values.
///
/// Swipes over the per-account cap (see [`SwipeCap`]) are dropped before
/// recording, and the request still succeeds.
///
/// [`SwipeCap`]: crate::inbound::http::swipe_cap::SwipeCap
///
/// # Errors
///
/// Returns 404 when the user has no metrics row, and 500 on a database failure.
pub async fn record_usage(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(batch): Json<HttpUsageBatch>,
) -> Result<StatusCode, ApiError> {
    let admitted = state.swipe_cap.admit(user.id, &batch, Instant::now());
    if let Some(overage) = admitted.warning {
        tracing::warn!(
            event = "usage_swipes_capped",
            user_id = %user.id,
            swipes_sent = overage.sent,
            swipes_kept = overage.kept,
            decisions_sent = overage.decisions_sent,
            decisions_kept = overage.decisions_kept,
            "usage flush over the swipe cap; swipes past it are dropped"
        );
    }
    let batch = admitted.batch;

    state
        .metrics_service
        .apply_usage(user.id, &batch)
        .await
        .map_err(ApiError::from)?;

    if has_swipes(&batch) {
        let user_id = user.id;
        let metrics = std::sync::Arc::clone(&state.metrics_service);
        tokio::spawn(async move {
            match metrics.mark_user_first_swiped(user_id).await {
                Ok(true) => {
                    if let Err(e) = metrics
                        .record_event(user_id, EventKind::FirstSwipe, None)
                        .await
                    {
                        tracing::warn!(error = ?e, "metrics: record first_swipe event failed");
                    }
                }
                Ok(false) => {}
                Err(e) => {
                    tracing::warn!(error = ?e, "metrics: mark_user_first_swiped failed");
                }
            }
        });
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Whether the batch carries any swipe, in any direction.
///
/// The counts are not summed, so four large ones cannot overflow.
fn has_swipes(batch: &HttpUsageBatch) -> bool {
    [
        batch.swipes_right,
        batch.swipes_left,
        batch.swipes_up,
        batch.swipes_down,
    ]
    .iter()
    .any(|&count| count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swipes_are_seen_whatever_the_direction() {
        let none = HttpUsageBatch::default();
        assert!(!has_swipes(&none));
        let up = HttpUsageBatch {
            swipes_up: 1,
            ..HttpUsageBatch::default()
        };
        assert!(has_swipes(&up));
    }

    #[test]
    fn huge_unclamped_counts_still_count_as_swipes() {
        let batch = HttpUsageBatch {
            swipes_right: u32::MAX,
            swipes_left: 1,
            ..HttpUsageBatch::default()
        };
        assert!(has_swipes(&batch));
    }
}
