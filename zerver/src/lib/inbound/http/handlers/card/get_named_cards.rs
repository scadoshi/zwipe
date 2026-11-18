//! Every card by its name's slug.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::slug::CardSlug;

use crate::domain::card::ports::CardService;
use crate::inbound::http::{ApiError, AppState};

/// The longest slug looked up, in characters: above the longest card name's slug, so a longer one
/// answers an empty list without a query.
const NAMED_SLUG_MAX: usize = 200;

/// Returns every card whose name, or front face's name, folds to the path's slug, one printing
/// each: a full-name match first, playable cards before tokens and art cards. A slug no card
/// has, text that is not a slug, or a slug longer than any card's answers an empty list.
///
/// # Errors
///
/// Returns 500 on a database failure.
pub async fn get_named_cards(
    State(state): State<AppState>,
    Path(name_slug): Path<String>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    if name_slug.len() > NAMED_SLUG_MAX {
        return Ok((StatusCode::OK, Json(vec![])));
    }
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
