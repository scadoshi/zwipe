//! Distinct language names handler.

use axum::Json;
use axum::extract::State;
use reqwest::StatusCode;

use crate::domain::card::requests::get_languages::GetLanguagesError;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<GetLanguagesError> for ApiError {
    fn from(value: GetLanguagesError) -> Self {
        match value {
            GetLanguagesError::Database(e) => e.to_500(),
        }
    }
}

/// Returns distinct language names.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn get_languages(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<String>>), ApiError> {
    state
        .card_service
        .fetch_languages()
        .await
        .map_err(ApiError::from)
        .map(|languages| (StatusCode::OK, Json(languages)))
}
