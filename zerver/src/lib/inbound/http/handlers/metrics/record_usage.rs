use axum::{Json, extract::State, http::StatusCode};

use crate::{
    domain::{
        auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
        health::ports::HealthService, metrics::models::errors::MetricsError,
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

    Ok(StatusCode::NO_CONTENT)
}
