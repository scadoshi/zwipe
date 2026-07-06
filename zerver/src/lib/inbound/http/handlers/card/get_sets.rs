#[cfg(feature = "zerver")]
use crate::{
    domain::card::requests::get_sets::GetSetsError,
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;

#[cfg(feature = "zerver")]
impl From<GetSetsError> for ApiError {
    fn from(value: GetSetsError) -> Self {
        match value {
            GetSetsError::Database(e) => e.log_500(),
        }
    }
}

/// Returns distinct set names.
#[cfg(feature = "zerver")]
pub async fn get_sets(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<String>>), ApiError> {
    state
        .card_service
        .get_sets()
        .await
        .map_err(ApiError::from)
        .map(|sets| (StatusCode::OK, Json(sets)))
}
