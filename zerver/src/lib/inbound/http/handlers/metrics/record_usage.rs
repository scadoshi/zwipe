use axum::{Json, extract::State, http::StatusCode};

use crate::{
    domain::{
        auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
        health::ports::HealthService,
        metrics::models::{errors::MetricsError, kinds::EventKind},
        user::ports::UserService,
    },
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};
use zwipe_core::http::contracts::metrics::HttpUsageBatch;

impl From<MetricsError> for ApiError {
    fn from(value: MetricsError) -> Self {
        match value {
            MetricsError::NotFound => Self::NotFound("metrics row not found".to_string()),
            MetricsError::Database(e) => e.log_500(),
        }
    }
}

/// Increments lifetime and daily counters for the caller by the batch values.
pub async fn record_usage<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(batch): Json<HttpUsageBatch>,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    state
        .metrics_service
        .apply_usage(user.id, &batch)
        .await
        .map_err(ApiError::from)?;

    let swiped =
        batch.swipes_right + batch.swipes_left + batch.swipes_up + batch.swipes_down > 0;
    if swiped {
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
