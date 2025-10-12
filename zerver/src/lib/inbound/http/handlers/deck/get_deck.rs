#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::{
                get_deck::{GetDeck, GetDeckError, GetDeckProfileError, InvalidGetDeck},
                Deck,
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

#[cfg(feature = "zerver")]
impl From<GetDeckProfileError> for ApiError {
    fn from(value: GetDeckProfileError) -> Self {
        match value {
            GetDeckProfileError::NotFound => Self::NotFound("deck profile not found".to_string()),
            GetDeckProfileError::Forbidden => {
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
    Path(deck_id): Path<String>,
    _: AuthenticatedUser,
) -> Result<(StatusCode, Json<Deck>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = GetDeck::new(user.id, &deck_id)?;

    state
        .deck_service
        .get_deck(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck| (StatusCode::OK, Json(deck)))
}
