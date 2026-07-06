#[cfg(feature = "zerver")]
use crate::{
    domain::card::requests::get_keywords::GetKeywordsError,
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;

#[cfg(feature = "zerver")]
impl From<GetKeywordsError> for ApiError {
    fn from(value: GetKeywordsError) -> Self {
        match value {
            GetKeywordsError::Database(e) => e.log_500(),
        }
    }
}

/// Returns distinct keyword ability names.
#[cfg(feature = "zerver")]
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
