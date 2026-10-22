use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;

use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState};

/// First-class commander search: same `CardQuery` body as the plain search, but
/// results are ordered by decks-helmed popularity, banded + wildcarded per user
/// per day, with token/emblem printings excluded. The shuffle seed is derived
/// from the authenticated user, so it needs no deck (works in create and edit).
/// (context/archive/commander_select_ordering.md)
pub async fn search_commanders(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<CardQuery>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    state
        .card_service
        .search_commanders(&body, user.id)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
