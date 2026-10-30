//! Distinct card type names handler.

use axum::Json;
use axum::extract::State;
use reqwest::StatusCode;

use crate::domain::card::requests::get_card_types::GetCardTypesError;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<GetCardTypesError> for ApiError {
    fn from(value: GetCardTypesError) -> Self {
        match value {
            GetCardTypesError::Database(e) => e.to_500(),
        }
    }
}

/// Returns distinct card type names.
pub async fn get_card_types(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<String>>), ApiError> {
    state
        .card_service
        .get_card_types()
        .await
        .map_err(ApiError::from)
        .map(|all_types| (StatusCode::OK, Json(all_types)))
}
