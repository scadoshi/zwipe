//! Import cards into a deck from plain-text decklist.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use zwipe_core::domain::deck::requests::import_deck_cards::{
    ImportDeckCards, ImportDeckCardsResult,
};
use zwipe_core::domain::user::requests::get_user::GetUser;
use zwipe_core::http::contracts::deck_card::HttpImportDeckCards;

use crate::domain::deck::models::deck_card::import_deck_cards::ImportDeckCardsError;
use crate::inbound::http::handlers::metrics::check_completion;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<ImportDeckCardsError> for ApiError {
    fn from(value: ImportDeckCardsError) -> Self {
        match value {
            ImportDeckCardsError::Forbidden => Self::NotFound("deck not found".to_string()),
            ImportDeckCardsError::DeckNotFound(e) => ApiError::from(e),
            ImportDeckCardsError::LimitReached => Self::UnprocessableEntity(
                "card limit reached (mainboard, maybeboard, and sideboard all count toward it)"
                    .to_string(),
            ),
            ImportDeckCardsError::UnverifiedLimitReached => Self::UnprocessableEntity(
                "card limit reached across all boards, verify your email to unlock more"
                    .to_string(),
            ),
            ImportDeckCardsError::Database(e) => e.to_500(),
        }
    }
}

/// Imports cards from plain-text decklist into a deck.
///
/// # Errors
///
/// Returns 422 for a malformed deck id or board or a full deck, 404 when the
/// deck isn't the caller's, and 500 on a database failure.
pub async fn import_deck_cards(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(deck_id): Path<String>,
    Json(body): Json<HttpImportDeckCards>,
) -> Result<(StatusCode, Json<ImportDeckCardsResult>), ApiError> {
    let deck_id = uuid::Uuid::try_parse(&deck_id)?;
    let db_user = state
        .user_service
        .fetch_user(&GetUser::from(user.id))
        .await?;
    let email_verified = db_user.email_verified_at.is_some();
    let board_override = body
        .board
        .as_deref()
        .map(zwipe_core::domain::deck::Board::try_from)
        .transpose()
        .map_err(|_| ApiError::UnprocessableEntity("invalid board value".to_string()))?;
    let request = ImportDeckCards::parse(
        user.id,
        deck_id,
        &body.text,
        email_verified,
        board_override,
        body.mode,
    );

    let result = state
        .deck_service
        .import_deck_cards(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let deck_service = std::sync::Arc::clone(&state.deck_service);
    let uid = user.id;
    tokio::spawn(check_completion::check_deck_completion(
        deck_service,
        metrics,
        uid,
        deck_id,
    ));

    Ok((StatusCode::OK, Json(result)))
}
