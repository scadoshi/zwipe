#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::{Path, State},
};

#[cfg(feature = "zerver")]
use crate::{
    domain::deck::models::deck::clear_deck_suppressions::ClearDeckSuppressionsError,
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::requests::clear_deck_suppressions::{
    ClearDeckSuppressions, InvalidClearDeckSuppressions,
};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck::HttpClearedSuppressions;

#[cfg(feature = "zerver")]
impl From<ClearDeckSuppressionsError> for ApiError {
    fn from(value: ClearDeckSuppressionsError) -> Self {
        match value {
            ClearDeckSuppressionsError::Database(e) => e.log_500(),
            ClearDeckSuppressionsError::Forbidden => {
                Self::NotFound("deck not found".to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidClearDeckSuppressions> for ApiError {
    fn from(value: InvalidClearDeckSuppressions) -> Self {
        match value {
            InvalidClearDeckSuppressions::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {}", e))
            }
        }
    }
}

/// Clears a deck's suppression set (skipped/removed cards) after ownership
/// verification, returning the number of rows removed.
#[cfg(feature = "zerver")]
pub async fn clear_deck_suppressions(
    State(state): State<AppState>,
    Path(deck_id): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<HttpClearedSuppressions>, ApiError> {
    let request = ClearDeckSuppressions::new(user.id, &deck_id)?;

    let cleared = state
        .deck_service
        .clear_deck_suppressions(&request)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(HttpClearedSuppressions { cleared }))
}
