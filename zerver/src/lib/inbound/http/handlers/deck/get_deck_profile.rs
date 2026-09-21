use crate::{
    domain::deck::models::deck::get_deck_profile::GetDeckProfileError,
    inbound::http::{ApiError, AppState, middleware::AuthenticatedUser},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;
use zwipe_core::domain::deck::{
    deck_profile::DeckProfile, requests::get_deck_profile::GetDeckProfile,
};

impl From<GetDeckProfileError> for ApiError {
    fn from(value: GetDeckProfileError) -> Self {
        use crate::inbound::http::To500;

        match value {
            GetDeckProfileError::Database(e) => e.to_500(),
            GetDeckProfileError::DeckProfileFromDb(e) => e.to_500(),
            GetDeckProfileError::Forbidden => Self::NotFound("deck not found".to_string()),
            GetDeckProfileError::NotFound => Self::NotFound("deck not found".to_string()),
        }
    }
}

/// Returns deck metadata with ownership verification.
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
