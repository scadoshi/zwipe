//! Idempotent (PATCH) deck-card update.
//!
//! Same fields as the PUT handler, but `quantity` is an **absolute** value
//! ("set to 3"), so replaying the request is harmless. The PUT delta route
//! stays registered until the version gate passes the first PATCH-speaking
//! client — migration: `context/plans/patch_idempotent_updates.md`.

#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck_card::HttpPatchDeckCard;

#[cfg(feature = "zerver")]
use crate::inbound::http::{
    ApiError, AppState, handlers::metrics::check_completion::check_deck_completion,
    middleware::AuthenticatedUser,
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::{DeckCard, requests::update_deck_card::UpdateDeckCard};

/// Sets a card's quantity (absolute), board, printing, and/or MVP star.
#[cfg(feature = "zerver")]
pub async fn patch_deck_card(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path((deck_id, scryfall_data_id)): Path<(String, String)>,
    Json(body): Json<HttpPatchDeckCard>,
) -> Result<(StatusCode, Json<DeckCard>), ApiError> {
    let board = body
        .board
        .as_deref()
        .map(zwipe_core::domain::deck::Board::try_from)
        .transpose()
        .map_err(|_| ApiError::UnprocessableEntity("invalid board value".to_string()))?;
    let request = UpdateDeckCard::patch(
        user.id,
        &deck_id,
        &scryfall_data_id,
        body.quantity,
        board,
        body.scryfall_data_id.as_deref(),
        body.mvp,
    )?;

    let deck_card = state
        .deck_service
        .update_deck_card(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    let deck_service = std::sync::Arc::clone(&state.deck_service);
    let uid = user.id;
    let did = request.deck_id;
    tokio::spawn(check_deck_completion(deck_service, metrics, uid, did));

    Ok((StatusCode::OK, Json(deck_card)))
}
