#[cfg(feature = "zerver")]
use crate::{
    domain::card::requests::get_artists::GetArtistsError,
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;

#[cfg(feature = "zerver")]
impl From<GetArtistsError> for ApiError {
    fn from(value: GetArtistsError) -> Self {
        match value {
            GetArtistsError::Database(e) => e.log_500(),
        }
    }
}

/// Returns distinct artist names.
#[cfg(feature = "zerver")]
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
