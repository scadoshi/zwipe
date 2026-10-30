//! Single card lookup handler.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use zwipe_core::domain::card::Card;

use crate::domain::card::requests::get_card::GetCardError;
use crate::domain::card::requests::get_card_profile::GetCardProfileError;
use crate::domain::card::requests::get_scryfall_data::{GetScryfallData, GetScryfallDataError};
use crate::inbound::http::{ApiError, AppState, To500};

impl From<GetCardProfileError> for ApiError {
    fn from(value: GetCardProfileError) -> Self {
        match value {
            GetCardProfileError::NotFound => Self::NotFound("card profile not found".to_string()),
            GetCardProfileError::CardProfileFromDb(e) | GetCardProfileError::Database(e) => {
                e.to_500()
            }
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
///
/// # Errors
///
/// Returns 422 for a malformed id, 404 when the card is missing, and 500 on a
/// database failure.
pub async fn get_card(
    State(state): State<AppState>,
    Path(scryfall_data_id): Path<String>,
) -> Result<(StatusCode, Json<Card>), ApiError> {
    let request = GetScryfallData::new(&scryfall_data_id)?;
    state
        .card_service
        .fetch_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|card| (StatusCode::OK, Json(card)))
}
