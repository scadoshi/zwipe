#[cfg(feature = "zerver")]
use crate::inbound::http::{ApiError, AppState, middleware::AuthenticatedUser};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::domain::card::{Card, search_card::card_filter::CardQuery};

/// First-class commander search: same `CardQuery` body as the plain search, but
/// results are ordered by decks-helmed popularity, banded + wildcarded per user
/// per day, with token/emblem printings excluded. The shuffle seed is derived
/// from the authenticated user, so it needs no deck (works in create and edit).
/// (context/archive/commander_select_ordering.md)
#[cfg(feature = "zerver")]
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
