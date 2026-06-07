#[cfg(feature = "zerver")]
use zwipe_core::domain::card::Card;
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{ApiError, AppState},
};
#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

/// Returns all printings of a card by oracle ID, ordered by release date.
#[cfg(feature = "zerver")]
pub async fn get_printings<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(oracle_id): Path<String>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let oracle_id = uuid::Uuid::try_parse(&oracle_id)
        .map_err(|e| ApiError::UnprocessableEntity(format!("invalid oracle id: {}", e)))?;
    state
        .card_service
        .get_printings(oracle_id)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
