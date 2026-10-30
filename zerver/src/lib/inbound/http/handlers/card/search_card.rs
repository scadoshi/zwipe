//! Card search handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;

use crate::domain::card::models::search_card::error::SearchCardsError;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<SearchCardsError> for ApiError {
    fn from(value: SearchCardsError) -> Self {
        value.to_500()
    }
}

/// Searches cards using a `CardQuery` deserialized from the JSON body.
///
/// # Errors
///
/// Returns 500 when the search fails.
pub async fn search_cards(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<CardQuery>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    state
        .card_service
        .search_cards(&body, user.id)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
