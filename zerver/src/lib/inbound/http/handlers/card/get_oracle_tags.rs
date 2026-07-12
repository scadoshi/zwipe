#[cfg(feature = "zerver")]
use crate::{
    domain::card::requests::get_oracle_tags::GetOracleTagsError,
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;
#[cfg(feature = "zerver")]
use zwipe_core::domain::card::oracle_tag::OracleTag;

#[cfg(feature = "zerver")]
impl From<GetOracleTagsError> for ApiError {
    fn from(value: GetOracleTagsError) -> Self {
        match value {
            GetOracleTagsError::Database(e) => e.log_500(),
        }
    }
}

/// Returns the full oracle tag catalog (slug, label, description, parent slugs).
#[cfg(feature = "zerver")]
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
