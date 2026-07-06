#[cfg(feature = "zerver")]
use crate::{
    domain::card::requests::get_languages::GetLanguagesError,
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;

#[cfg(feature = "zerver")]
impl From<GetLanguagesError> for ApiError {
    fn from(value: GetLanguagesError) -> Self {
        match value {
            GetLanguagesError::Database(e) => e.log_500(),
        }
    }
}

/// Returns distinct language names.
#[cfg(feature = "zerver")]
pub async fn get_languages(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<String>>), ApiError> {
    state
        .card_service
        .get_languages()
        .await
        .map_err(ApiError::from)
        .map(|languages| (StatusCode::OK, Json(languages)))
}
