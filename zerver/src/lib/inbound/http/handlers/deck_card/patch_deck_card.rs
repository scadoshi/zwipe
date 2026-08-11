//! Idempotent (PATCH) deck-card update.
//!
//! `quantity` is an **absolute** value ("set to 3"), so replaying the
//! request is harmless. The legacy PUT delta route was removed once the
//! version gate passed the PATCH-speaking clients — migration:
//! `context/plans/patch_idempotent_updates.md`.
//!
//! Every field of a deck card is non-clearable, so an explicit `null` in
//! the body is a 422 per the RFC 7396 resolution (null clears nullable
//! fields, errors on required ones); absent means untouched.

#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[cfg(feature = "zerver")]
use zwipe_core::http::{contracts::deck_card::HttpPatchDeckCard, helpers::Opdate};

#[cfg(feature = "zerver")]
use crate::{
    domain::deck::models::deck_card::update_deck_card::UpdateDeckCardError,
    inbound::http::{
        ApiError, AppState, To500, handlers::metrics::check_completion::check_deck_completion,
        middleware::AuthenticatedUser,
    },
};
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::{
    DeckCard,
    requests::update_deck_card::{InvalidUpdateDeckCard, UpdateDeckCard},
};

#[cfg(feature = "zerver")]
impl From<UpdateDeckCardError> for ApiError {
    fn from(value: UpdateDeckCardError) -> Self {
        match value {
            UpdateDeckCardError::QuantityUnderflow => {
                Self::UnprocessableEntity("resulting quantity cannot be zero or less".to_string())
            }
            UpdateDeckCardError::NotFound => {
                Self::UnprocessableEntity("deck card not found".to_string())
            }
            // Verbatim client-facing copy — the app shows this in a toast.
            UpdateDeckCardError::MvpCapReached => {
                Self::UnprocessableEntity("This deck already has 3 MVPs".to_string())
            }
            UpdateDeckCardError::MvpNotMainboard => {
                Self::UnprocessableEntity("Only cards in the deck can be MVPs".to_string())
            }
            UpdateDeckCardError::Database(e) => e.to_500(),
            UpdateDeckCardError::DeckCardFromDb(e) => e.to_500(),
            UpdateDeckCardError::GetDeckProfileError(e) => ApiError::from(e),
            UpdateDeckCardError::Forbidden => {
                Self::Forbidden(UpdateDeckCardError::Forbidden.to_string())
            }
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidUpdateDeckCard> for ApiError {
    fn from(value: InvalidUpdateDeckCard) -> Self {
        match value {
            InvalidUpdateDeckCard::DeckId(e) => {
                Self::UnprocessableEntity(format!("invalid deck id: {}", e))
            }
            InvalidUpdateDeckCard::ScryfallDataId(e) => {
                Self::UnprocessableEntity(format!("invalid card id: {}", e))
            }
            InvalidUpdateDeckCard::Quantity(e) => {
                Self::UnprocessableEntity(format!("invalid quantity: {}", e))
            }
            InvalidUpdateDeckCard::NewScryfallDataId(e) => {
                Self::UnprocessableEntity(format!("invalid printing id: {}", e))
            }
            InvalidUpdateDeckCard::NothingToUpdate => {
                Self::UnprocessableEntity(InvalidUpdateDeckCard::NothingToUpdate.to_string())
            }
        }
    }
}

/// Flattens a non-clearable field: absent passes through as no-op, a value
/// passes through as set, and explicit `null` is a 422 (the field always
/// has a value, so "clear" is meaningless).
#[cfg(feature = "zerver")]
pub fn reject_null<T>(field: Opdate<T>, name: &str) -> Result<Option<T>, ApiError> {
    match field {
        Opdate::Unchanged => Ok(None),
        Opdate::Set(Some(value)) => Ok(Some(value)),
        Opdate::Set(None) => Err(ApiError::UnprocessableEntity(format!(
            "{name} cannot be null"
        ))),
    }
}

/// Sets a card's quantity (absolute), board, printing, and/or MVP star.
#[cfg(feature = "zerver")]
pub async fn patch_deck_card(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path((deck_id, scryfall_data_id)): Path<(String, String)>,
    Json(body): Json<HttpPatchDeckCard>,
) -> Result<(StatusCode, Json<DeckCard>), ApiError> {
    let quantity = reject_null(body.quantity, "quantity")?;
    let board = reject_null(body.board, "board")?;
    let printing = reject_null(body.scryfall_data_id, "scryfall_data_id")?;
    let mvp = reject_null(body.mvp, "mvp")?;

    let board = board
        .as_deref()
        .map(zwipe_core::domain::deck::Board::try_from)
        .transpose()
        .map_err(|_| ApiError::UnprocessableEntity("invalid board value".to_string()))?;
    let request = UpdateDeckCard::patch(
        user.id,
        &deck_id,
        &scryfall_data_id,
        quantity,
        board,
        printing.as_deref(),
        mvp,
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
