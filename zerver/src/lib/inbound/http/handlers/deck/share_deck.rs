//! Deck share/unshare handlers (owner-side token management).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;
use zwipe_core::domain::deck::requests::get_deck_profile::GetDeckProfile;
use zwipe_core::http::contracts::deck::HttpDeckShareToken;

use crate::domain::deck::models::deck::share_deck::ShareDeckError;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<ShareDeckError> for ApiError {
    fn from(value: ShareDeckError) -> Self {
        match value {
            ShareDeckError::Database(e) => e.to_500(),
            ShareDeckError::Forbidden => Self::NotFound("deck not found".to_string()),
            ShareDeckError::NotFound => Self::NotFound(ShareDeckError::NotFound.to_string()),
        }
    }
}

/// Generates (or regenerates) the deck's share token after ownership
/// verification. Re-sharing rotates the token, so any old link dies.
///
/// # Errors
///
/// Returns 404 when the deck is missing or isn't the caller's, and 500 on a
/// database failure.
pub async fn share_deck(
    State(state): State<AppState>,
    Path(deck_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<(StatusCode, Json<HttpDeckShareToken>), ApiError> {
    let request = GetDeckProfile::new(user.id, deck_id);

    let share_token = state
        .deck_service
        .share_deck(&request)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(HttpDeckShareToken { share_token })))
}

/// Revokes the deck's share token after ownership verification. The public
/// link 404s from here on (modulo a short CF cache window).
///
/// # Errors
///
/// Returns 404 when the deck is missing or isn't the caller's, and 500 on a
/// database failure.
pub async fn unshare_deck(
    State(state): State<AppState>,
    Path(deck_id): Path<Uuid>,
    user: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .unshare_deck(&request)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
