#[cfg(feature = "zerver")]
use crate::{
    domain::card::requests::get_oracle_words::GetOracleWordsError,
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;

#[cfg(feature = "zerver")]
impl From<GetOracleWordsError> for ApiError {
    fn from(value: GetOracleWordsError) -> Self {
        match value {
            GetOracleWordsError::Database(e) => e.log_500(),
        }
    }
}

/// Returns distinct normalized words extracted from oracle text.
#[cfg(feature = "zerver")]
pub async fn get_oracle_words(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<String>>), ApiError> {
    state
        .card_service
        .get_oracle_words()
        .await
        .map_err(ApiError::from)
        .map(|all_words| (StatusCode::OK, Json(all_words)))
}
