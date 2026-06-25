#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::update_deck_profile::UpdateDeckProfileError,
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{
        handlers::metrics::check_completion::check_deck_completion,
        middleware::AuthenticatedUser, ApiError, AppState, Log500,
    },
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::{
    deck_profile::DeckProfile,
    requests::update_deck_profile::{InvalidUpdateDeckProfile, UpdateDeckProfile},
};
#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
impl From<UpdateDeckProfileError> for ApiError {
    fn from(value: UpdateDeckProfileError) -> Self {
        match value {
            UpdateDeckProfileError::NotFound => {
                Self::NotFound("deck profile not found".to_string())
            }
            UpdateDeckProfileError::Duplicate => Self::UnprocessableEntity(
                "deck with name and user combination already exists".to_string(),
            ),
            UpdateDeckProfileError::GetDeckProfileError(e) => ApiError::from(e),
            UpdateDeckProfileError::DeckFromDb(e) => e.log_500(),
            UpdateDeckProfileError::Database(e) => e.log_500(),
            UpdateDeckProfileError::Forbidden => {
                Self::Forbidden(UpdateDeckProfileError::Forbidden.to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidUpdateDeckProfile> for ApiError {
    fn from(value: InvalidUpdateDeckProfile) -> Self {
        match value {
            InvalidUpdateDeckProfile::DeckName(e) => {
                Self::UnprocessableEntity(format!("invalid deck name: {}", e))
            }
            InvalidUpdateDeckProfile::Format(e) => {
                Self::UnprocessableEntity(format!("invalid format: {}", e))
            }
            InvalidUpdateDeckProfile::DeckTag(e) => {
                Self::UnprocessableEntity(format!("invalid deck tag: {}", e))
            }
            InvalidUpdateDeckProfile::TooManyTags => {
                Self::UnprocessableEntity("a deck may have at most 5 tags".to_string())
            }
            InvalidUpdateDeckProfile::NoUpdates => {
                Self::UnprocessableEntity("must update at least one field".to_string())
            }
        }
    }
}

/// Updates deck metadata with ownership verification.
#[cfg(feature = "zerver")]
pub async fn update_deck_profile<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(deck_id): Path<Uuid>,
    Json(body): Json<HttpUpdateDeckProfile>,
) -> Result<(StatusCode, Json<DeckProfile>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let format_raw: Option<Option<String>> = body.format.into_option();
    let format_option: Option<Option<&str>> = format_raw
        .as_ref()
        .map(|opt| opt.as_deref());
    let request = UpdateDeckProfile::builder(deck_id, user.id)
        .name(body.name.as_deref())
        .commander_id(body.commander_id.into_option())
        .partner_commander_id(body.partner_commander_id.into_option())
        .background_id(body.background_id.into_option())
        .signature_spell_id(body.signature_spell_id.into_option())
        .format(format_option)
        .tags(body.tags.into_option())
        .build()?;

    let deck_profile = state
        .deck_service
        .update_deck_profile(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let deck_service = std::sync::Arc::clone(&state.deck_service);
    let uid = user.id;
    tokio::spawn(check_deck_completion(deck_service, metrics, uid, deck_id));

    Ok((StatusCode::OK, Json(deck_profile)))
}
