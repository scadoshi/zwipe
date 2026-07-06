#[cfg(feature = "zerver")]
use crate::{
    domain::deck::models::deck::get_deck::GetDeckError,
    inbound::http::{ApiError, AppState, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::Deck;

#[cfg(feature = "zerver")]
impl From<GetDeckError> for ApiError {
    fn from(value: GetDeckError) -> Self {
        match value {
            GetDeckError::GetCardError(e) => ApiError::from(e),
            GetDeckError::GetDeckCardError(e) => ApiError::from(e),
            GetDeckError::GetCardProfileError(e) => ApiError::from(e),
            GetDeckError::GetDeckProfileError(e) => ApiError::from(e),
        }
    }
}

/// Returns the full deck including all cards (not just metadata like `get_deck_profile`).
#[cfg(feature = "zerver")]
pub async fn get_deck(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(deck_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Deck>), ApiError> {
    use zwipe_core::domain::deck::requests::get_deck_profile::GetDeckProfile;

    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .get_deck(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck| (StatusCode::OK, Json(deck)))
}
