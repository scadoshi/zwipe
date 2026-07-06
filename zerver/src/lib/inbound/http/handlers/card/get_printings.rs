#[cfg(feature = "zerver")]
use crate::inbound::http::{ApiError, AppState};
#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::card::Card;

/// Returns all printings of a card by oracle ID, ordered by release date.
#[cfg(feature = "zerver")]
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
