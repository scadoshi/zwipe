//! Distinct artist names handler.

use axum::Json;
use axum::extract::State;
use reqwest::StatusCode;

use crate::domain::card::requests::get_artists::GetArtistsError;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<GetArtistsError> for ApiError {
    fn from(value: GetArtistsError) -> Self {
        match value {
            GetArtistsError::Database(e) => e.to_500(),
        }
    }
}

/// Returns distinct artist names.
pub async fn get_artists(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<String>>), ApiError> {
    state
        .card_service
        .get_artists()
        .await
        .map_err(ApiError::from)
        .map(|artists| (StatusCode::OK, Json(artists)))
}
