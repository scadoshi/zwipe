//! Every card by its name's slug.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::slug::CardSlug;

use crate::domain::card::ports::CardService;
use crate::inbound::http::{ApiError, AppState};

/// Returns every card whose name, or front face's name, folds to the path's slug, one printing
/// each: a full-name match first, playable cards before tokens and art cards. A slug no card
/// has, or text that is not a slug, answers an empty list.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn get_named_cards(
    State(state): State<AppState>,
    Path(name_slug): Path<String>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    let Ok(slug) = name_slug.parse::<CardSlug>() else {
        return Ok((StatusCode::OK, Json(vec![])));
    };
    state
        .card_service
        .fetch_named_cards(&slug)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}
