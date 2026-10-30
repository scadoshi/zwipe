//! Distinct oracle text word names handler.

use axum::Json;
use axum::extract::State;
use reqwest::StatusCode;

use crate::domain::card::requests::get_oracle_words::GetOracleWordsError;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<GetOracleWordsError> for ApiError {
    fn from(value: GetOracleWordsError) -> Self {
        match value {
            GetOracleWordsError::Database(e) => e.to_500(),
        }
    }
}

/// Returns distinct normalized words extracted from oracle text.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn get_oracle_words(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<String>>), ApiError> {
    state
        .card_service
        .fetch_oracle_words()
        .await
        .map_err(ApiError::from)
        .map(|all_words| (StatusCode::OK, Json(all_words)))
}
