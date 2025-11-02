#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::{
            models::{
                search_card::{InvalidSearchCards, SearchCards, SearchCardsError},
                Card,
            },
            ports::CardService,
        },
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};

#[cfg(feature = "zerver")]
impl From<SearchCardsError> for ApiError {
    fn from(value: SearchCardsError) -> Self {
        value.log_500()
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidSearchCards> for ApiError {
    fn from(value: InvalidSearchCards) -> Self {
        Self::UnprocessableEntity(format!("invalid request: {}", value))
    }
}

#[cfg(feature = "zerver")]
pub async fn search_cards<AS, US, HS, CS, DS>(
    _: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<SearchCards>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    state
        .card_service
        .search_cards(&body)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
