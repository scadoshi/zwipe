#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck_card::delete_deck_card::{
                DeleteDeckCard, DeleteDeckCardError, InvalidDeleteDeckCard,
            },
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};

#[cfg(feature = "zerver")]
impl From<DeleteDeckCardError> for ApiError {
    fn from(value: DeleteDeckCardError) -> Self {
        match value {
            DeleteDeckCardError::NotFound => {
                Self::UnprocessableEntity("deck card not found".to_string())
            }
            DeleteDeckCardError::Database(e) => e.log_500(),
            DeleteDeckCardError::GetDeckProfileError(e) => ApiError::from(e),
            DeleteDeckCardError::Forbidden => {
                Self::Forbidden(DeleteDeckCardError::Forbidden.to_string())
            }
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
            InvalidDeleteDeckCard::ScryfallDataId(e) => {
                Self::UnprocessableEntity(format!("invalid card profile id: {}", e))
            }
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn delete_deck_card<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path((deck_id, scryfall_data_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = DeleteDeckCard::new(user.id, &deck_id, &scryfall_data_id)?;

    state
        .deck_service
        .delete_deck_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::NO_CONTENT)
}
