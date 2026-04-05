//! Import cards into a deck from plain-text decklist.

#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck_card::HttpImportDeckCards;
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::requests::get_user::GetUser;

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck_card::import_deck_cards::ImportDeckCardsError,
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::requests::import_deck_cards::{ImportDeckCards, ImportDeckCardsResult};

#[cfg(feature = "zerver")]
impl From<ImportDeckCardsError> for ApiError {
    fn from(value: ImportDeckCardsError) -> Self {
        match value {
            ImportDeckCardsError::Forbidden => {
                Self::Forbidden(ImportDeckCardsError::Forbidden.to_string())
            }
            ImportDeckCardsError::DeckNotFound(e) => ApiError::from(e),
            ImportDeckCardsError::LimitReached => {
                Self::UnprocessableEntity("card limit reached — verify your email to unlock more".to_string())
            }
            ImportDeckCardsError::Database(e) => e.log_500(),
        }
    }
}

/// Imports cards from plain-text decklist into a deck.
#[cfg(feature = "zerver")]
pub async fn import_deck_cards<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(deck_id): Path<String>,
    Json(body): Json<HttpImportDeckCards>,
) -> Result<(StatusCode, Json<ImportDeckCardsResult>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let deck_id = uuid::Uuid::try_parse(&deck_id)?;
    let db_user = state.user_service.get_user(&GetUser::from(user.id)).await?;
    let email_verified = db_user.email_verified_at.is_some();
    let request = ImportDeckCards::parse(user.id, deck_id, &body.text, email_verified);

    state
        .deck_service
        .import_deck_cards(&request)
        .await
        .map_err(ApiError::from)
        .map(|result| (StatusCode::OK, Json(result)))
}
