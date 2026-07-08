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
    domain::deck::models::deck::share_deck::GetSharedDeckError,
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck::HttpSharedDeck;

#[cfg(feature = "zerver")]
impl From<GetSharedDeckError> for ApiError {
    fn from(value: GetSharedDeckError) -> Self {
        match value {
            GetSharedDeckError::Database(e) => e.log_500(),
            GetSharedDeckError::NotFound => {
                Self::NotFound(GetSharedDeckError::NotFound.to_string())
            }
            GetSharedDeckError::GetDeck(e) => ApiError::from(e),
        }
    }
}

/// Public shared-deck read — no auth; possession of the token is the
/// authority. Revoked or never-issued tokens 404. Responses may be CF-cached
/// briefly (~5 min): a shared deck updating a few minutes late is fine, and a
/// revoked token dying a few minutes late is acceptable.
///
/// The response strips owner identity — a malformed token is answered exactly
/// like a revoked one, so the endpoint never confirms what exists.
#[cfg(feature = "zerver")]
pub async fn get_shared_deck(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<(StatusCode, Json<HttpSharedDeck>), ApiError> {
    let token = Uuid::try_parse(&token)
        .map_err(|_| ApiError::NotFound(GetSharedDeckError::NotFound.to_string()))?;

    let shared = state
        .deck_service
        .get_shared_deck(token)
        .await
        .map_err(ApiError::from)?;

    let body = HttpSharedDeck {
        name: shared.deck.deck_profile.name.to_string(),
        format: shared.deck.deck_profile.format,
        power_level: shared.deck.deck_profile.power_level,
        tags: shared.deck.deck_profile.tags,
        other_tags: shared.deck.deck_profile.other_tags,
        commander: shared.commander,
        partner_commander: shared.partner_commander,
        background: shared.background,
        signature_spell: shared.signature_spell,
        entries: shared.deck.entries,
    };

    Ok((StatusCode::OK, Json(body)))
}
