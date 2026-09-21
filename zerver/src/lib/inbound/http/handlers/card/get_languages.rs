use crate::{
    domain::card::requests::get_languages::GetLanguagesError,
    inbound::http::{ApiError, AppState, To500},
};
use axum::{Json, extract::State};
use reqwest::StatusCode;

impl From<GetLanguagesError> for ApiError {
    fn from(value: GetLanguagesError) -> Self {
        match value {
            GetLanguagesError::Database(e) => e.to_500(),
        }
    }
}

/// Returns distinct language names.
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
