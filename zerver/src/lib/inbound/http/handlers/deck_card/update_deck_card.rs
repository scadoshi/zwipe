#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck_card::HttpUpdateDeckCard;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck_card::update_deck_card::UpdateDeckCardError,
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
    DeckCard,
    requests::update_deck_card::{InvalidUpdateDeckCard, UpdateDeckCard},
};

#[cfg(feature = "zerver")]
impl From<UpdateDeckCardError> for ApiError {
    fn from(value: UpdateDeckCardError) -> Self {
        match value {
            UpdateDeckCardError::QuantityUnderflow => Self::UnprocessableEntity(
                "resulting quantity cannot be zero or less".to_string(),
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
            InvalidUpdateDeckCard::ScryfallDataId(e) => {
                Self::UnprocessableEntity(format!("invalid card id: {}", e))
            }
            InvalidUpdateDeckCard::UpdateQuantity(e) => {
                Self::UnprocessableEntity(format!("invalid update quantity: {}", e))
            }
            InvalidUpdateDeckCard::NewScryfallDataId(e) => {
                Self::UnprocessableEntity(format!("invalid printing id: {}", e))
            }
            InvalidUpdateDeckCard::NothingToUpdate => {
                Self::UnprocessableEntity(InvalidUpdateDeckCard::NothingToUpdate.to_string())
            }
        }
    }
}

/// Updates a card's quantity, board, and/or printing.
#[cfg(feature = "zerver")]
pub async fn update_deck_card<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path((deck_id, scryfall_data_id)): Path<(String, String)>,
    Json(body): Json<HttpUpdateDeckCard>,
) -> Result<(StatusCode, Json<DeckCard>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let board = body.board.as_deref().map(zwipe_core::domain::deck::Board::try_from).transpose().map_err(|_| ApiError::UnprocessableEntity("invalid board value".to_string()))?;
    let request = UpdateDeckCard::new(user.id, &deck_id, &scryfall_data_id, body.update_quantity, board, body.scryfall_data_id.as_deref())?;

    let deck_card = state
        .deck_service
        .update_deck_card(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let deck_service = std::sync::Arc::clone(&state.deck_service);
    let uid = user.id;
    let did = request.deck_id;
    tokio::spawn(check_deck_completion(deck_service, metrics, uid, did));

    Ok((StatusCode::OK, Json(deck_card)))
}
