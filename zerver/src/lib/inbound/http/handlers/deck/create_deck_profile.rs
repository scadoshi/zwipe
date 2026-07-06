#[cfg(feature = "zerver")]
use crate::{
    domain::{
        deck::models::deck::create_deck_profile::CreateDeckProfileError,
        metrics::models::kinds::EventKind,
    },
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::{
    deck_profile::DeckProfile,
    requests::create_deck_profile::{CreateDeckProfile, InvalidCreateDeckProfile},
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::requests::get_user::GetUser;
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck::HttpCreateDeckProfile;

#[cfg(feature = "zerver")]
impl From<CreateDeckProfileError> for ApiError {
    fn from(value: CreateDeckProfileError) -> Self {
        match value {
            CreateDeckProfileError::Duplicate => Self::UnprocessableEntity(
                "deck with name and user combination already exists".to_string(),
            ),
            CreateDeckProfileError::LimitReached => {
                Self::UnprocessableEntity("deck limit reached".to_string())
            }
            CreateDeckProfileError::UnverifiedLimitReached => Self::UnprocessableEntity(
                "deck limit reached, verify your email to unlock more".to_string(),
            ),
            CreateDeckProfileError::Database(e) => e.log_500(),
            CreateDeckProfileError::DeckFromDb(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidCreateDeckProfile> for ApiError {
    fn from(value: InvalidCreateDeckProfile) -> Self {
        match value {
            InvalidCreateDeckProfile::DeckName(e) => {
                Self::UnprocessableEntity(format!("invalid deck name: {}", e))
            }
            InvalidCreateDeckProfile::Format(e) => {
                Self::UnprocessableEntity(format!("invalid format: {}", e))
            }
            InvalidCreateDeckProfile::DeckTag(e) => {
                Self::UnprocessableEntity(format!("invalid deck tag: {}", e))
            }
            InvalidCreateDeckProfile::TooManyTags => {
                Self::UnprocessableEntity("a deck may have at most 5 tags".to_string())
            }
            InvalidCreateDeckProfile::PowerLevel(e) => {
                Self::UnprocessableEntity(format!("invalid power level: {}", e))
            }
            InvalidCreateDeckProfile::DeckOtherTag(e) => {
                Self::UnprocessableEntity(format!("invalid other-tag: {}", e))
            }
            InvalidCreateDeckProfile::TooManyOtherTags => {
                Self::UnprocessableEntity("a deck may have at most 5 other-tags".to_string())
            }
        }
    }
}

/// Creates a new deck for the authenticated user.
#[cfg(feature = "zerver")]
pub async fn create_deck_profile(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<HttpCreateDeckProfile>,
) -> Result<(StatusCode, Json<DeckProfile>), ApiError> {
    let db_user = state.user_service.get_user(&GetUser::from(user.id)).await?;
    let email_verified = db_user.email_verified_at.is_some();
    let request = CreateDeckProfile::builder(body.name, user.id, email_verified)
        .commander_id(body.commander_id)
        .partner_commander_id(body.partner_commander_id)
        .background_id(body.background_id)
        .signature_spell_id(body.signature_spell_id)
        .format(body.format.as_deref())
        .tags(body.tags.unwrap_or_default())
        .power_level(body.power_level.as_deref())
        .other_tags(body.other_tags.unwrap_or_default())
        .land_target(body.land_target)
        .price_target(body.price_target)
        .price_target_currency(body.price_target_currency)
        .build()?;

    let deck_profile = state
        .deck_service
        .create_deck_profile(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let uid = user.id;
    let deck_id = deck_profile.id;
    tokio::spawn(async move {
        if let Err(e) = metrics.increment_decks_created(uid).await {
            tracing::warn!(error = ?e, "metrics: increment decks_created failed");
        }
        if let Err(e) = metrics
            .record_event(uid, EventKind::DeckCreated, Some(deck_id))
            .await
        {
            tracing::warn!(error = ?e, "metrics: record deck_created event failed");
        }
    });

    Ok((StatusCode::CREATED, Json(deck_profile)))
}
