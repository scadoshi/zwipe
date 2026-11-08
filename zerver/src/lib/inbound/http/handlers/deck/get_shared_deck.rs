//! Public shared-deck read handler (token-addressed, no auth).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;
use zwipe_core::domain::user::theme::ThemeConfig;
use zwipe_core::http::contracts::deck::HttpSharedDeck;

use crate::domain::deck::models::deck::share_deck::GetSharedDeckError;
use crate::domain::deck::ports::DeckService;
use crate::domain::user::ports::UserService;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<GetSharedDeckError> for ApiError {
    fn from(value: GetSharedDeckError) -> Self {
        match value {
            GetSharedDeckError::Database(e) => e.to_500(),
            GetSharedDeckError::NotFound => {
                Self::NotFound(GetSharedDeckError::NotFound.to_string())
            }
            GetSharedDeckError::GetDeck(e) => ApiError::from(e),
        }
    }
}

/// Public shared-deck read, no auth: possession of the token is the
/// authority. Revoked or never-issued tokens 404. Responses may be CF-cached
/// briefly (~5 min): a shared deck updating a few minutes late is fine, and a
/// revoked token dying a few minutes late is acceptable.
///
/// The response strips owner identity: a malformed token is answered exactly
/// like a revoked one, so the endpoint never confirms what exists.
///
/// # Errors
///
/// Returns 404 for a malformed, revoked or unknown token, and 500 on a database
/// failure.
pub async fn get_shared_deck(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<(StatusCode, Json<HttpSharedDeck>), ApiError> {
    let token = share_token(&token)?;
    Ok((StatusCode::OK, Json(shared_body(&state, token).await?)))
}

/// `token` as a share token, or the same 404 a revoked token gets.
///
/// # Errors
///
/// Returns 404 when `token` isn't a UUID.
pub(crate) fn share_token(token: &str) -> Result<Uuid, ApiError> {
    Uuid::try_parse(token).map_err(|_| ApiError::NotFound(GetSharedDeckError::NotFound.to_string()))
}

/// The shared deck `token` names, as the public share routes send it: no
/// owner identity, but the owner's theme, which only styles the preview
/// image. A settings lookup that fails leaves the theme out rather than
/// failing the page.
///
/// # Errors
///
/// Returns 404 for a revoked or unknown token, and 500 on a database failure.
pub(crate) async fn shared_body(state: &AppState, token: Uuid) -> Result<HttpSharedDeck, ApiError> {
    let shared = state
        .deck_service
        .fetch_shared_deck(token)
        .await
        .map_err(ApiError::from)?;
    let theme = match state
        .user_service
        .fetch_settings(shared.deck.deck_profile.user_id)
        .await
    {
        Ok(settings) => Some(ThemeConfig::from(&settings)),
        Err(e) => {
            tracing::warn!(error = %e, "shared deck: owner settings did not load");
            None
        }
    };
    let profile = shared.deck.deck_profile;
    Ok(HttpSharedDeck {
        name: profile.name.to_string(),
        format: profile.format,
        power_level: profile.power_level,
        tags: profile.tags,
        other_tags: profile.other_tags,
        oracle_tags: profile.oracle_tags,
        commander: shared.commander,
        partner_commander: shared.partner_commander,
        background: shared.background,
        signature_spell: shared.signature_spell,
        entries: shared.deck.entries,
        tokens: shared.tokens,
        land_target: profile.land_target,
        price_target: profile.price_target,
        price_target_currency: profile.price_target_currency,
        updated_at: profile.updated_at,
        theme,
    })
}
