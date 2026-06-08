#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::requests::get_user::GetUser;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck_card::create_deck_card::CreateDeckCardError,
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
    requests::create_deck_card::{CreateDeckCard, InvalidCreateDeckCard},
};

#[cfg(feature = "zerver")]
impl From<CreateDeckCardError> for ApiError {
    fn from(value: CreateDeckCardError) -> Self {
        match value {
            CreateDeckCardError::Duplicate => {
                Self::UnprocessableEntity("card and deck combination already exist".to_string())
            }
            CreateDeckCardError::IsCommander => {
                Self::UnprocessableEntity(CreateDeckCardError::IsCommander.to_string())
            }
            CreateDeckCardError::LimitReached => {
                Self::UnprocessableEntity("card limit reached — verify your email to unlock more".to_string())
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
            InvalidCreateDeckCard::OracleId(e) => {
                Self::UnprocessableEntity(format!("invalid oracle id: {}", e))
            }
            InvalidCreateDeckCard::Quantity(e) => {
                Self::UnprocessableEntity(format!("invalid quantity: {}", e))
            }
        }
    }
}

/// Adds a card to a deck with the specified quantity.
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
    let db_user = state.user_service.get_user(&GetUser::from(user.id)).await?;
    let email_verified = db_user.email_verified_at.is_some();
    let board = body.board.as_deref().map(zwipe_core::domain::deck::Board::try_from).transpose().map_err(|_| ApiError::UnprocessableEntity("invalid board value".to_string()))?;
    let request = CreateDeckCard::new(user.id, &deck_id, &body.scryfall_data_id, &body.oracle_id, body.quantity, board, email_verified)?;

    let deck_card = state
        .deck_service
        .create_deck_card(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let deck_service = std::sync::Arc::clone(&state.deck_service);
    let uid = user.id;
    let did = request.deck_id;
    tokio::spawn(check_deck_completion(deck_service, metrics, uid, did));

    Ok((StatusCode::CREATED, Json(deck_card)))
}
