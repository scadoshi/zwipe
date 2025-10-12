#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::create_deck_profile::{
                CreateDeckProfile, CreateDeckProfileError, InvalidCreateDeckProfile,
            },
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{
        handlers::deck::HttpDeckProfile, middleware::AuthenticatedUser, ApiError, AppState, Log500,
    },
};
#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

#[cfg(feature = "zerver")]
impl From<CreateDeckProfileError> for ApiError {
    fn from(value: CreateDeckProfileError) -> Self {
        match value {
            CreateDeckProfileError::Duplicate => Self::UnprocessableEntity(
                "deck with name and user combination already exists".to_string(),
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
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpCreateDeckProfile {
    pub name: String,
}

impl HttpCreateDeckProfile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn create_deck_profile<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpCreateDeckProfile>,
) -> Result<(StatusCode, Json<HttpDeckProfile>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = CreateDeckProfile::new(&body.name, user.id)?;

    state
        .deck_service
        .create_deck_profile(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_profile| (StatusCode::CREATED, Json(deck_profile.into())))
}
