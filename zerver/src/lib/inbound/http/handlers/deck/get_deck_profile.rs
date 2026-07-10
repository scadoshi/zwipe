#[cfg(feature = "zerver")]
use crate::{
    domain::deck::models::deck::get_deck_profile::GetDeckProfileError,
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
use zwipe_core::domain::deck::{
    deck_profile::DeckProfile, requests::get_deck_profile::GetDeckProfile,
};

#[cfg(feature = "zerver")]
impl From<GetDeckProfileError> for ApiError {
    fn from(value: GetDeckProfileError) -> Self {
        use crate::inbound::http::Log500;

        match value {
            GetDeckProfileError::Database(e) => e.log_500(),
            GetDeckProfileError::DeckProfileFromDb(e) => e.log_500(),
            GetDeckProfileError::Forbidden => {
                Self::NotFound("deck not found".to_string())
            }
            GetDeckProfileError::NotFound => Self::NotFound("deck not found".to_string()),
        }
    }
}

/// Returns deck metadata with ownership verification.
#[cfg(feature = "zerver")]
pub async fn get_deck_profile(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(deck_id): Path<Uuid>,
) -> Result<(StatusCode, Json<DeckProfile>), ApiError> {
    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .get_deck_profile(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_profile| (StatusCode::OK, Json(deck_profile)))
}
