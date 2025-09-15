#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck_card::{
                CreateDeckCard, CreateDeckCardError, DeckCard, DeleteDeckCard, DeleteDeckCardError,
                InvalidCreateDeckCard, InvalidDeleteDeckCard, InvalidUpdateDeckCard,
                UpdateDeckCard, UpdateDeckCardError,
            },
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};

// ===========
//  http types
// ===========

#[derive(Debug, Serialize)]
pub struct HttpDeckCard {
    pub deck_id: String,
    pub card_profile_id: String,
    pub quantity: i32,
}

impl HttpDeckCard {
    pub fn new(deck_id: &str, card_profile_id: &str, quantity: i32) -> Self {
        Self {
            deck_id: deck_id.to_string(),
            card_profile_id: card_profile_id.to_string(),
            quantity,
        }
    }
}

impl From<DeckCard> for HttpDeckCard {
    fn from(value: DeckCard) -> Self {
        Self {
            deck_id: value.deck_id.to_string(),
            card_profile_id: value.card_profile_id.to_string(),
            quantity: value.quantity.quantity(),
        }
    }
}

// ========
//  create
// ========

#[cfg(feature = "zerver")]
impl From<CreateDeckCardError> for ApiError {
    fn from(value: CreateDeckCardError) -> Self {
        match value {
            CreateDeckCardError::Duplicate => {
                Self::UnprocessableEntity("card and deck combination already exist".to_string())
            }
            e => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidCreateDeckCard> for ApiError {
    fn from(value: InvalidCreateDeckCard) -> Self {
        match value {
            InvalidCreateDeckCard::CardProfileId(e) => {
                Self::UnprocessableEntity(format!("invalid card id: {}", e))
            }
            InvalidCreateDeckCard::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {}", e))
            }
            InvalidCreateDeckCard::Quantity(e) => {
                Self::UnprocessableEntity(format!("invalid quantity: {}", e))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpCreateDeckCard {
    card_profile_id: String,
    quantity: i32,
}

impl HttpCreateDeckCard {
    pub fn new(card_profile_id: &str, quantity: i32) -> Self {
        Self {
            card_profile_id: card_profile_id.to_string(),
            quantity,
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn create_deck_card<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(deck_id): Path<String>,
    Json(body): Json<HttpCreateDeckCard>,
) -> Result<(StatusCode, Json<HttpDeckCard>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = CreateDeckCard::new(&deck_id, &body.card_profile_id, body.quantity, user.id)?;

    state
        .deck_service
        .create_deck_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_card| (StatusCode::CREATED, Json(deck_card.into())))
}

// ========
//  update
// ========

#[cfg(feature = "zerver")]
impl From<UpdateDeckCardError> for ApiError {
    fn from(value: UpdateDeckCardError) -> Self {
        match value {
            UpdateDeckCardError::InvalidResultingQuantity => Self::UnprocessableEntity(
                "resulting quantity must remain greater than 0".to_string(),
            ),
            UpdateDeckCardError::NotFound => {
                Self::UnprocessableEntity("deck card not found".to_string())
            }
            e => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidUpdateDeckCard> for ApiError {
    fn from(value: InvalidUpdateDeckCard) -> Self {
        match value {
            InvalidUpdateDeckCard::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {}", e))
            }
            InvalidUpdateDeckCard::CardProfileId(e) => {
                Self::UnprocessableEntity(format!("invalid card id: {}", e))
            }
            InvalidUpdateDeckCard::UpdateQuantity(e) => {
                Self::UnprocessableEntity(format!("invalid update quantity: {}", e))
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HttpUpdateDeckCard {
    update_quantity: i32,
}

impl HttpUpdateDeckCard {
    pub fn new(update_quantity: i32) -> Self {
        Self { update_quantity }
    }
}

#[cfg(feature = "zerver")]
pub async fn update_deck_card<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path((deck_id, card_profile_id)): Path<(String, String)>,
    Json(body): Json<HttpUpdateDeckCard>,
) -> Result<(StatusCode, Json<HttpDeckCard>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = UpdateDeckCard::new(&deck_id, &card_profile_id, body.update_quantity, user.id)?;

    state
        .deck_service
        .update_deck_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_card| (StatusCode::OK, Json(deck_card.into())))
}

// ========
//  delete
// ========

#[cfg(feature = "zerver")]
impl From<DeleteDeckCardError> for ApiError {
    fn from(value: DeleteDeckCardError) -> Self {
        match value {
            DeleteDeckCardError::NotFound => {
                Self::UnprocessableEntity("deck card not found".to_string())
            }
            e => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidDeleteDeckCard> for ApiError {
    fn from(value: InvalidDeleteDeckCard) -> Self {
        match value {
            InvalidDeleteDeckCard::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {}", e))
            }
            InvalidDeleteDeckCard::CardProfileId(e) => {
                Self::UnprocessableEntity(format!("invalid card profile id: {}", e))
            }
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn delete_deck_card<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path((deck_id, card_profile_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = DeleteDeckCard::new(&deck_id, &card_profile_id, user.id)?;

    state
        .deck_service
        .delete_deck_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::NO_CONTENT)
}
