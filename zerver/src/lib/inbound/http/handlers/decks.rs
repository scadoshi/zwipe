use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        auth::ports::AuthService,
        card::{
            models::{
                card_profile::GetCardProfileError, scryfall_data::GetScryfallDataError,
                GetCardError,
            },
            ports::CardService,
        },
        deck::{
            models::{
                deck::{
                    CreateDeckProfile, CreateDeckProfileError, Deck, DeckProfile, DeleteDeck,
                    DeleteDeckError, GetDeck, GetDeckError, GetDeckProfileError,
                    InvalidCreateDeckProfile, InvalidGetDeck, InvalidUpdateDeckProfile,
                    UpdateDeckProfile, UpdateDeckProfileError,
                },
                deck_card::GetDeckCardError,
            },
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{ApiError, ApiSuccess, AppState, Log500},
};

#[derive(Debug, Serialize, PartialEq)]
pub struct HttpDeckProfile {
    id: String,
    name: String,
    user_id: String,
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

impl From<CreateDeckProfileError> for ApiError {
    fn from(value: CreateDeckProfileError) -> Self {
        match value {
            CreateDeckProfileError::Duplicate => Self::UnprocessableEntity(
                "deck with name and user combination already exists".to_string(),
            ),
            e => e.log_500(),
        }
    }
}

impl From<InvalidCreateDeckProfile> for ApiError {
    fn from(value: InvalidCreateDeckProfile) -> Self {
        match value {
            InvalidCreateDeckProfile::DeckName(e) => {
                Self::UnprocessableEntity(format!("invalid deck name: {}", e))
            }
            InvalidCreateDeckProfile::UserId(e) => {
                Self::UnprocessableEntity(format!("invalid user id: {}", e))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpCreateDeckProfile {
    pub name: String,
    pub user_id: String,
}

impl TryFrom<HttpCreateDeckProfile> for CreateDeckProfile {
    type Error = InvalidCreateDeckProfile;
    fn try_from(value: HttpCreateDeckProfile) -> Result<Self, Self::Error> {
        CreateDeckProfile::new(&value.name, &value.user_id)
    }
}

pub async fn create_deck_profile<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpCreateDeckProfile>,
) -> Result<ApiSuccess<HttpDeckProfile>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = CreateDeckProfile::new(&body.name, &body.user_id)?;

    state
        .deck_service
        .create_deck_profile(&request)
        .await
        .map_err(ApiError::from)
        .map(|ref deck_profile| ApiSuccess::new(StatusCode::OK, deck_profile.into()))
}

// =====
//  get
// =====

impl From<GetDeckError> for ApiError {
    fn from(value: GetDeckError) -> Self {
        match value {
            GetDeckError::GetCardError(gce) => match gce {
                GetCardError::GetCardProfileError(gcpe) => match gcpe {
                    GetCardProfileError::NotFound => {
                        Self::NotFound("card profile not found".to_string())
                    }
                    e => e.log_500(),
                },
                GetCardError::GetScryfallDataError(gsfde) => match gsfde {
                    GetScryfallDataError::NotFound => {
                        Self::NotFound("scryfall data not found".to_string())
                    }
                    e => e.log_500(),
                },
            },

            GetDeckError::GetCardProfileError(gcpe) => match gcpe {
                GetCardProfileError::NotFound => {
                    Self::NotFound("card profile not found".to_string())
                }
                e => e.log_500(),
            },

            GetDeckError::GetDeckCardError(gdce) => match gdce {
                GetDeckCardError::NotFound => Self::NotFound("deck card not found".to_string()),
                e => e.log_500(),
            },

            GetDeckError::GetDeckProfileError(gdpe) => match gdpe {
                GetDeckProfileError::NotFound => {
                    Self::NotFound("deck profile not found".to_string())
                }
                e => e.log_500(),
            },
        }
    }
}

impl From<InvalidGetDeck> for ApiError {
    fn from(value: InvalidGetDeck) -> Self {
        match value {
            InvalidGetDeck::Id(e) => Self::UnprocessableEntity(format!("invalid id: {}", e)),
        }
    }
}

pub async fn get_deck<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(id): Path<String>,
) -> Result<ApiSuccess<Deck>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = GetDeck::new(&id)?;

    state
        .deck_service
        .get_deck(&request)
        .await
        .map_err(ApiError::from)
        .map(|ref deck| ApiSuccess::new(StatusCode::OK, deck.clone()))
}

// ========
//  update
// ========

impl From<UpdateDeckProfileError> for ApiError {
    fn from(value: UpdateDeckProfileError) -> Self {
        match value {
            UpdateDeckProfileError::NotFound => {
                Self::NotFound("deck profile not found".to_string())
            }
            UpdateDeckProfileError::Duplicate => Self::UnprocessableEntity(
                "deck with name and user combination already exists".to_string(),
            ),
            e => e.log_500(),
        }
    }
}

impl From<InvalidUpdateDeckProfile> for ApiError {
    fn from(value: InvalidUpdateDeckProfile) -> Self {
        match value {
            InvalidUpdateDeckProfile::InvalidDeckName(e) => {
                Self::UnprocessableEntity(format!("invalid deck name: {}", e))
            }
            InvalidUpdateDeckProfile::InvalidId(e) => {
                Self::UnprocessableEntity(format!("invalid id: {}", e))
            }
            InvalidUpdateDeckProfile::NothingToUpdate => {
                Self::UnprocessableEntity("must update at least one field".to_string())
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpUpdateDeckProfileBody {
    name: Option<String>,
}

pub async fn update_deck_profile<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(id): Path<String>,
    Json(body): Json<HttpUpdateDeckProfileBody>,
) -> Result<ApiSuccess<HttpDeckProfile>, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = UpdateDeckProfile::new(&id, body.name.as_deref())?;

    state
        .deck_service
        .update_deck_profile(&request)
        .await
        .map_err(ApiError::from)
        .map(|ref deck_profile| ApiSuccess::new(StatusCode::OK, deck_profile.into()))
}

// ========
//  delete
// ========

impl From<DeleteDeckError> for ApiError {
    fn from(value: DeleteDeckError) -> Self {
        match value {
            DeleteDeckError::NotFound => Self::NotFound("deck not found".to_string()),
            e => e.log_500(),
        }
    }
}

pub async fn delete_deck<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(id): Path<String>,
) -> Result<ApiSuccess<()>, ApiError>
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
        .map(|_| ApiSuccess::new(StatusCode::OK, ()))
}
