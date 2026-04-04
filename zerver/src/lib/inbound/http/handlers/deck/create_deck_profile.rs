#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::create_deck_profile::CreateDeckProfileError,
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
    requests::create_deck_profile::{CreateDeckProfile, InvalidCreateDeckProfile},
};
#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use zwipe_core::http::contracts::deck::HttpCreateDeckProfile;
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::requests::get_user::GetUser;

#[cfg(feature = "zerver")]
impl From<CreateDeckProfileError> for ApiError {
    fn from(value: CreateDeckProfileError) -> Self {
        match value {
            CreateDeckProfileError::Duplicate => Self::UnprocessableEntity(
                "deck with name and user combination already exists".to_string(),
            ),
            CreateDeckProfileError::LimitReached => {
                Self::UnprocessableEntity("deck limit reached — verify your email to unlock more".to_string())
            }
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
        }
    }
}

/// Creates a new deck for the authenticated user.
#[cfg(feature = "zerver")]
pub async fn create_deck_profile<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpCreateDeckProfile>,
) -> Result<(StatusCode, Json<DeckProfile>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let db_user = state.user_service.get_user(&GetUser::from(user.id)).await?;
    let email_verified = db_user.email_verified_at.is_some();
    let request = CreateDeckProfile::new(body.name, body.commander_id, body.format.as_deref(), user.id, email_verified)?;

    state
        .deck_service
        .create_deck_profile(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_profile| (StatusCode::CREATED, Json(deck_profile)))
}
