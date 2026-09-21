use crate::{
    domain::card::requests::{
        get_card::GetCardError,
        get_card_profile::GetCardProfileError,
        get_scryfall_data::{GetScryfallData, GetScryfallDataError},
    },
    inbound::http::{ApiError, AppState, To500},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use zwipe_core::domain::card::Card;

impl From<GetCardProfileError> for ApiError {
    fn from(value: GetCardProfileError) -> Self {
        match value {
            GetCardProfileError::NotFound => Self::NotFound("card profile not found".to_string()),
            GetCardProfileError::CardProfileFromDb(e) => e.to_500(),
            GetCardProfileError::Database(e) => e.to_500(),
        }
    }
}

impl From<GetScryfallDataError> for ApiError {
    fn from(value: GetScryfallDataError) -> Self {
        match value {
            GetScryfallDataError::NotFound => Self::NotFound("scryfall data not found".to_string()),
            GetScryfallDataError::Database(e) => e.to_500(),
        }
    }
}

impl From<GetCardError> for ApiError {
    fn from(value: GetCardError) -> Self {
        match value {
            GetCardError::GetCardProfileError(e) => ApiError::from(e),
            GetCardError::GetScryfallDataError(e) => ApiError::from(e),
        }
    }
}

/// Returns a single card by Scryfall data ID.
pub async fn get_card(
    State(state): State<AppState>,
    Path(scryfall_data_id): Path<String>,
) -> Result<(StatusCode, Json<Card>), ApiError> {
    let request = GetScryfallData::new(&scryfall_data_id)?;
    state
        .card_service
        .get_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|card| (StatusCode::OK, Json(card)))
}
