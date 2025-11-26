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
                create_deck_card::{CreateDeckCard, CreateDeckCardError, InvalidCreateDeckCard},
                DeckCard,
            },
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};

#[cfg(feature = "zerver")]
impl From<CreateDeckCardError> for ApiError {
    fn from(value: CreateDeckCardError) -> Self {
        match value {
            CreateDeckCardError::Duplicate => {
                Self::UnprocessableEntity("card and deck combination already exist".to_string())
            }
            CreateDeckCardError::Database(e) => e.log_500(),
            CreateDeckCardError::DeckCardFromDb(e) => e.log_500(),
            CreateDeckCardError::GetDeckProfileError(e) => ApiError::from(e),
            CreateDeckCardError::Forbidden => {
                Self::Forbidden(CreateDeckCardError::Forbidden.to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidCreateDeckCard> for ApiError {
    fn from(value: InvalidCreateDeckCard) -> Self {
        match value {
            InvalidCreateDeckCard::ScryfallDataId(e) => {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpCreateDeckCard {
    pub scryfall_data_id: String,
    pub quantity: i32,
}

impl HttpCreateDeckCard {
    pub fn new(scryfall_data_id: &str, quantity: i32) -> Self {
        Self {
            scryfall_data_id: scryfall_data_id.to_string(),
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
) -> Result<(StatusCode, Json<DeckCard>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = CreateDeckCard::new(user.id, &deck_id, &body.scryfall_data_id, body.quantity)?;

    state
        .deck_service
        .create_deck_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_card| (StatusCode::CREATED, Json(deck_card)))
}
