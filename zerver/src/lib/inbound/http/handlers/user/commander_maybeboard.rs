//! Commander maybeboard handlers (per-user "maybe this commander" list).

#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

#[cfg(feature = "zerver")]
use crate::{
    domain::deck::models::deck::commander_maybeboard::CommanderMaybeboardError,
    inbound::http::{ApiError, AppState, To500, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::card::Card;
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::requests::commander_maybeboard::{
    CommanderMaybeboardCard, InvalidCommanderMaybeboardCard,
};

#[cfg(feature = "zerver")]
impl From<CommanderMaybeboardError> for ApiError {
    fn from(value: CommanderMaybeboardError) -> Self {
        match value {
            CommanderMaybeboardError::Database(e) => e.to_500(),
            CommanderMaybeboardError::UnknownCard => Self::NotFound("card not found".to_string()),
            CommanderMaybeboardError::LimitReached => {
                Self::UnprocessableEntity("commander maybeboard is full".to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidCommanderMaybeboardCard> for ApiError {
    fn from(value: InvalidCommanderMaybeboardCard) -> Self {
        match value {
            InvalidCommanderMaybeboardCard::OracleId(e) => {
                Self::UnprocessableEntity(format!("invalid oracle id: {}", e))
            }
        }
    }
}

/// Returns the authenticated user's commander maybeboard, hydrated to full
/// cards (preferred printing), newest save first.
#[cfg(feature = "zerver")]
pub async fn get_commander_maybeboard(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    state
        .deck_service
        .get_commander_maybeboard(user.id)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}

/// Adds a commander to the authenticated user's maybeboard (idempotent;
/// duplicate = no-op success, over-cap rejected).
#[cfg(feature = "zerver")]
pub async fn add_commander_maybeboard_card(
    State(state): State<AppState>,
    Path(oracle_id): Path<String>,
    user: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let request = CommanderMaybeboardCard::from_path(user.id, &oracle_id)?;

    state
        .deck_service
        .add_commander_maybeboard_card(&request)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Deletes the authenticated user's entire commander maybeboard (idempotent).
#[cfg(feature = "zerver")]
pub async fn clear_commander_maybeboard(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    state
        .deck_service
        .clear_commander_maybeboard(user.id)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Removes a commander from the authenticated user's maybeboard (idempotent).
#[cfg(feature = "zerver")]
pub async fn remove_commander_maybeboard_card(
    State(state): State<AppState>,
    Path(oracle_id): Path<String>,
    user: AuthenticatedUser,
) -> Result<StatusCode, ApiError> {
    let request = CommanderMaybeboardCard::from_path(user.id, &oracle_id)?;

    state
        .deck_service
        .remove_commander_maybeboard_card(&request)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
