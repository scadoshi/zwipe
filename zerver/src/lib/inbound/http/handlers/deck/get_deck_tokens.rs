//! Get tokens produced by a deck's cards.

#[cfg(feature = "zerver")]
use zwipe_core::domain::card::Card;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::get_deck_tokens::GetDeckTokensError,
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState},
};
#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
impl From<GetDeckTokensError> for ApiError {
    fn from(value: GetDeckTokensError) -> Self {
        match value {
            GetDeckTokensError::GetDeckError(e) => ApiError::from(e),
            GetDeckTokensError::GetCardError(e) => ApiError::from(e),
        }
    }
}

/// Returns all token cards produced by the cards in a deck.
#[cfg(feature = "zerver")]
pub async fn get_deck_tokens<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(deck_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    use zwipe_core::domain::deck::requests::get_deck_profile::GetDeckProfile;

    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .get_deck_tokens(&request)
        .await
        .map_err(ApiError::from)
        .map(|tokens| (StatusCode::OK, Json(tokens)))
}
