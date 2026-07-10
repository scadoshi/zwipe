#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
};

#[cfg(feature = "zerver")]
use crate::{
    domain::deck::models::deck::delete_deck::DeleteDeckError,
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::requests::delete_deck::{DeleteDeck, InvalidDeleteDeck};

#[cfg(feature = "zerver")]
impl From<DeleteDeckError> for ApiError {
    fn from(value: DeleteDeckError) -> Self {
        match value {
            DeleteDeckError::NotFound => Self::NotFound("deck not found".to_string()),
            DeleteDeckError::Database(e) => e.log_500(),
            DeleteDeckError::Forbidden => Self::NotFound("deck not found".to_string()),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidDeleteDeck> for ApiError {
    fn from(value: InvalidDeleteDeck) -> Self {
        match value {
            InvalidDeleteDeck::UserId(e) => {
                Self::UnprocessableEntity(format!("invalid user id: {}", e))
            }
            InvalidDeleteDeck::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {}", e))
            }
        }
    }
}

/// Deletes a deck after ownership verification.
#[cfg(feature = "zerver")]
pub async fn delete_deck(
    State(state): State<AppState>,
    Path(deck_id): Path<String>,
    user: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let request = DeleteDeck::new(user.id, &deck_id)?;

    state
        .deck_service
        .delete_deck(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::NO_CONTENT)
}
