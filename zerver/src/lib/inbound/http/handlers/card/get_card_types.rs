#[cfg(feature = "zerver")]
use crate::{
    domain::card::requests::get_card_types::GetCardTypesError,
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;

#[cfg(feature = "zerver")]
impl From<GetCardTypesError> for ApiError {
    fn from(value: GetCardTypesError) -> Self {
        match value {
            GetCardTypesError::Database(e) => e.log_500(),
        }
    }
}

/// Returns distinct card type names.
#[cfg(feature = "zerver")]
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
