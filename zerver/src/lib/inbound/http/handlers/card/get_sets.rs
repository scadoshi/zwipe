use crate::{
    domain::card::requests::get_sets::GetSetsError,
    inbound::http::{ApiError, AppState, To500},
};
use axum::{Json, extract::State};
use reqwest::StatusCode;

impl From<GetSetsError> for ApiError {
    fn from(value: GetSetsError) -> Self {
        match value {
            GetSetsError::Database(e) => e.to_500(),
        }
    }
}

/// Returns distinct set names.
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
