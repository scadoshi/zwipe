use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use zwipe_core::domain::card::Card;

use crate::inbound::http::{ApiError, AppState};

/// Returns all printings of a card by oracle ID, ordered by release date.
pub async fn get_printings(
    State(state): State<AppState>,
    Path(oracle_id): Path<String>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    let oracle_id = uuid::Uuid::try_parse(&oracle_id)
        .map_err(|e| ApiError::UnprocessableEntity(format!("invalid oracle id: {}", e)))?;
    state
        .card_service
        .get_printings(oracle_id)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
