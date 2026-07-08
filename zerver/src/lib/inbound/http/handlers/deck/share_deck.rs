#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
use crate::{
    domain::deck::models::deck::share_deck::ShareDeckError,
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::requests::get_deck_profile::GetDeckProfile;
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck::HttpDeckShareToken;

#[cfg(feature = "zerver")]
impl From<ShareDeckError> for ApiError {
    fn from(value: ShareDeckError) -> Self {
        match value {
            ShareDeckError::Database(e) => e.log_500(),
            ShareDeckError::Forbidden => Self::Forbidden(ShareDeckError::Forbidden.to_string()),
            ShareDeckError::NotFound => Self::NotFound(ShareDeckError::NotFound.to_string()),
        }
    }
}

/// Generates (or regenerates) the deck's share token after ownership
/// verification. Re-sharing rotates the token, so any old link dies.
#[cfg(feature = "zerver")]
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
#[cfg(feature = "zerver")]
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
