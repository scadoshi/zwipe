//! Oracle tag catalog handler.

use axum::Json;
use axum::extract::State;
use reqwest::StatusCode;
use zwipe_core::domain::card::oracle_tag::OracleTag;

use crate::domain::card::requests::get_oracle_tags::GetOracleTagsError;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<GetOracleTagsError> for ApiError {
    fn from(value: GetOracleTagsError) -> Self {
        match value {
            GetOracleTagsError::Database(e) => e.to_500(),
        }
    }
}

/// Returns the full oracle tag catalog (slug, label, description, parent slugs).
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn get_oracle_tags(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<OracleTag>>), ApiError> {
    state
        .card_service
        .get_oracle_tags()
        .await
        .map_err(ApiError::from)
        .map(|all_oracle_tags| (StatusCode::OK, Json(all_oracle_tags)))
}
