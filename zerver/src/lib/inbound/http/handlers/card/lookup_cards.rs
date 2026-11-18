//! Card lookup handler: the card search a signed-out caller may make.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::HttpCardLookup;

use crate::domain::card::models::search_card::error::LookupCardsError;
use crate::domain::card::ports::CardService;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<LookupCardsError> for ApiError {
    fn from(value: LookupCardsError) -> Self {
        value.to_500()
    }
}

/// Answers a [`HttpCardLookup`] with at most 30 cards, one per card. The body's own checks
/// answer 422 before this runs. A token, when sent, applies the user's Universes Beyond setting.
///
/// # Errors
///
/// Returns 401 for a token that is sent but isn't valid, and 500 when the search fails.
pub async fn lookup_cards(
    user: Option<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(lookup): Json<HttpCardLookup>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    state
        .card_service
        .lookup_cards(&lookup, user.map(|u| u.id))
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
