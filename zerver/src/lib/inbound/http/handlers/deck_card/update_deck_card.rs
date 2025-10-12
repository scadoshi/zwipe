#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck_card::update_deck_card::{
                InvalidUpdateDeckCard, UpdateDeckCard, UpdateDeckCardError,
            },
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{
        handlers::deck_card::HttpDeckCard, middleware::AuthenticatedUser, ApiError, AppState,
        Log500,
    },
};

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
            UpdateDeckCardError::Database(e) => e.log_500(),
            UpdateDeckCardError::DeckCardFromDb(e) => e.log_500(),
            UpdateDeckCardError::GetDeckProfileError(e) => ApiError::from(e),
            UpdateDeckCardError::Forbidden => {
                Self::Forbidden(UpdateDeckCardError::Forbidden.to_string())
            }
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
    let request = UpdateDeckCard::new(user.id, &deck_id, &card_profile_id, body.update_quantity)?;

    state
        .deck_service
        .update_deck_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_card| (StatusCode::OK, Json(deck_card.into())))
}
