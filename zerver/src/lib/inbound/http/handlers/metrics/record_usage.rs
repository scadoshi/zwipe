//! Accepts a batched usage update from the client.

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
/// # Errors
///
/// Returns 404 when the user has no metrics row, and 500 on a database failure.
pub async fn record_usage(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(batch): Json<HttpUsageBatch>,
) -> Result<StatusCode, ApiError> {
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
/// The batch is not clamped yet, so the counts are not summed: four large
/// ones would overflow.
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
