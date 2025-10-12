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
            models::deck::delete_deck::{DeleteDeck, DeleteDeckError, InvalidDeleteDeck},
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};

#[cfg(feature = "zerver")]
impl From<DeleteDeckError> for ApiError {
    fn from(value: DeleteDeckError) -> Self {
        match value {
            DeleteDeckError::NotFound => Self::NotFound("deck not found".to_string()),
            DeleteDeckError::Database(e) => e.log_500(),
            DeleteDeckError::Forbidden => Self::Forbidden(DeleteDeckError::Forbidden.to_string()),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidDeleteDeck> for ApiError {
    fn from(value: InvalidDeleteDeck) -> Self {
        match value {
            InvalidDeleteDeck::UserId(e) => {
                Self::UnprocessableEntity(format!("invalid user id: {}", e))
            }
            InvalidDeleteDeck::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {}", e))
            }
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn delete_deck<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(deck_id): Path<String>,
    user: AuthenticatedUser,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = DeleteDeck::new(user.id, &deck_id)?;

    state
        .deck_service
        .delete_deck(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::NO_CONTENT)
}
