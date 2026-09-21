use crate::{
    domain::card::requests::get_keywords::GetKeywordsError,
    inbound::http::{ApiError, AppState, To500},
};
use axum::{Json, extract::State};
use reqwest::StatusCode;

impl From<GetKeywordsError> for ApiError {
    fn from(value: GetKeywordsError) -> Self {
        match value {
            GetKeywordsError::Database(e) => e.to_500(),
        }
    }
}

/// Returns distinct keyword ability names.
pub async fn get_keywords(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<String>>), ApiError> {
    state
        .card_service
        .get_keywords()
        .await
        .map_err(ApiError::from)
        .map(|all_keywords| (StatusCode::OK, Json(all_keywords)))
}
