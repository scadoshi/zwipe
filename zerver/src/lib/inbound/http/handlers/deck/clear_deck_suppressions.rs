//! Deck suppression-clear handler.

use axum::Json;
use axum::extract::{Path, State};
use zwipe_core::domain::deck::requests::clear_deck_suppressions::{
    ClearDeckSuppressions, InvalidClearDeckSuppressions,
};
use zwipe_core::http::contracts::deck::HttpClearedSuppressions;

use crate::domain::deck::models::deck::clear_deck_suppressions::ClearDeckSuppressionsError;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<ClearDeckSuppressionsError> for ApiError {
    fn from(value: ClearDeckSuppressionsError) -> Self {
        match value {
            ClearDeckSuppressionsError::Database(e) => e.to_500(),
            ClearDeckSuppressionsError::Forbidden => Self::NotFound("deck not found".to_string()),
        }
    }
}

impl From<InvalidClearDeckSuppressions> for ApiError {
    fn from(value: InvalidClearDeckSuppressions) -> Self {
        match value {
            InvalidClearDeckSuppressions::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {e}"))
            }
        }
    }
}

/// Clears a deck's suppression set (skipped/removed cards) after ownership
/// verification, returning the number of rows removed.
///
/// # Errors
///
/// Returns 422 for a malformed deck id, 404 when the deck isn't the caller's,
/// and 500 on a database failure.
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
