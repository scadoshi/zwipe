use crate::domain::deck::models::deck::DeckProfile;
#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::{
    Deck, InvalidCreateDeckProfile, InvalidGetDeck, InvalidUpdateDeckProfile,
};
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::{
                CreateDeckProfile, CreateDeckProfileError, DeleteDeck, DeleteDeckError, GetDeck,
                GetDeckError, GetDeckProfileError, UpdateDeckProfile, UpdateDeckProfileError,
            },
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

// ============
//  http types
// ============

#[derive(Debug, Serialize, PartialEq)]
pub struct HttpDeckProfile {
    id: String,
    name: String,
    user_id: String,
}

impl HttpDeckProfile {
    pub fn new(id: &str, name: &str, user_id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            user_id: user_id.to_string(),
        }
    }
}

impl From<DeckProfile> for HttpDeckProfile {
    fn from(value: DeckProfile) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name.to_string(),
            user_id: value.user_id.to_string(),
        }
    }
}

impl From<&DeckProfile> for HttpDeckProfile {
    fn from(value: &DeckProfile) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name.to_string(),
            user_id: value.user_id.to_string(),
        }
    }
}

// ========
//  create
// ========

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

// =====
//  get
// =====

#[cfg(feature = "zerver")]
impl From<GetDeckProfileError> for ApiError {
    fn from(value: GetDeckProfileError) -> Self {
        match value {
            GetDeckProfileError::NotFound => Self::NotFound("deck profile not found".to_string()),
            GetDeckProfileError::DeckNotOwnedByUser => {
                Self::Forbidden("deck does not belong to the requesting user".to_string())
            }
            GetDeckProfileError::DeckProfileFromDb(e) => e.log_500(),
            GetDeckProfileError::Database(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<GetDeckError> for ApiError {
    fn from(value: GetDeckError) -> Self {
        match value {
            GetDeckError::GetCardError(e) => ApiError::from(e),
            GetDeckError::GetDeckCardError(e) => ApiError::from(e),
            GetDeckError::GetCardProfileError(e) => ApiError::from(e),
            GetDeckError::GetDeckProfileError(e) => ApiError::from(e),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidGetDeck> for ApiError {
    fn from(value: InvalidGetDeck) -> Self {
        match value {
            InvalidGetDeck::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {}", e))
            }
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn get_deck<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(id): Path<String>,
    _: AuthenticatedUser,
) -> Result<(StatusCode, Json<Deck>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = GetDeck::new(&id, &user.id)?;

    state
        .deck_service
        .get_deck(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck| (StatusCode::OK, Json(deck)))
}

// =====================
//  update deck profile
// =====================

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
            InvalidUpdateDeckProfile::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid id: {}", e))
            }
            InvalidUpdateDeckProfile::NoUpdates => {
                Self::UnprocessableEntity("must update at least one field".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpUpdateDeckProfileBody {
    name: Option<String>,
}

impl HttpUpdateDeckProfileBody {
    pub fn new(name: Option<&str>) -> Self {
        Self {
            name: name.map(|name| name.to_string()),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn update_deck_profile<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(deck_id): Path<String>,
    Json(body): Json<HttpUpdateDeckProfileBody>,
) -> Result<(StatusCode, Json<HttpDeckProfile>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = UpdateDeckProfile::new(&deck_id, body.name.as_deref(), user.id)?;

    state
        .deck_service
        .update_deck_profile(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_profile| (StatusCode::OK, Json(deck_profile.into())))
}

// ========
//  delete
// ========

#[cfg(feature = "zerver")]
impl From<DeleteDeckError> for ApiError {
    fn from(value: DeleteDeckError) -> Self {
        match value {
            DeleteDeckError::NotFound => Self::NotFound("deck not found".to_string()),
            DeleteDeckError::Database(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn delete_deck<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(id): Path<String>,
    _: AuthenticatedUser,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = DeleteDeck::new(&id)?;

    state
        .deck_service
        .delete_deck(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::NO_CONTENT)
}
