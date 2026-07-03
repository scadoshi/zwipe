#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{models::deck::skip_deck_card::SkipDeckCardError, ports::DeckService},
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::requests::skip_deck_card::{InvalidSkipDeckCard, SkipDeckCard};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck::HttpSkipDeckCard;

#[cfg(feature = "zerver")]
impl From<SkipDeckCardError> for ApiError {
    fn from(value: SkipDeckCardError) -> Self {
        match value {
            SkipDeckCardError::Database(e) => e.log_500(),
            SkipDeckCardError::Forbidden => {
                Self::Forbidden(SkipDeckCardError::Forbidden.to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidSkipDeckCard> for ApiError {
    fn from(value: InvalidSkipDeckCard) -> Self {
        match value {
            InvalidSkipDeckCard::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {}", e))
            }
            InvalidSkipDeckCard::OracleId(e) => {
                Self::UnprocessableEntity(format!("invalid oracle id: {}", e))
            }
        }
    }
}

/// Suppresses a single card for a deck (durable skip) after ownership
/// verification.
#[cfg(feature = "zerver")]
pub async fn skip_deck_card<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(deck_id): Path<String>,
    user: AuthenticatedUser,
    Json(body): Json<HttpSkipDeckCard>,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = SkipDeckCard::new(user.id, &deck_id, body.oracle_id)?;

    state
        .deck_service
        .skip_deck_card(&request)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Removes a single skip suppression (undo) after ownership verification.
#[cfg(feature = "zerver")]
pub async fn unskip_deck_card<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path((deck_id, oracle_id)): Path<(String, String)>,
    user: AuthenticatedUser,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = SkipDeckCard::from_path(user.id, &deck_id, &oracle_id)?;

    state
        .deck_service
        .unskip_deck_card(&request)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
