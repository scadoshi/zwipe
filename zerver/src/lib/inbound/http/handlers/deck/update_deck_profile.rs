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
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
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
pub use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
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
    let request = UpdateDeckProfile::new(
        deck_id,
        body.name.as_deref(),
        body.commander_id.into_option(),
        format_option,
        user.id,
    )?;

    state
        .deck_service
        .update_deck_profile(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_profile| (StatusCode::OK, Json(deck_profile)))
}
