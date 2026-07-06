//! Import an Archidekt deck's card list into an existing deck.
//!
//! The client sends the deck URL; the server extracts the id, fetches the
//! deck from Archidekt's public API, resolves each printing by Scryfall id,
//! and imports the cards into the caller's deck exactly like the plain-text
//! importer (same boards, same add/replace modes, same result shape).
//! Keeping fetch + parse server-side means the (undocumented) Archidekt shape
//! can be patched without an app release.

#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCardsResult;
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::requests::get_user::GetUser;
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::deck::HttpImportArchidektDeck;

#[cfg(feature = "zerver")]
use crate::{
    inbound::http::{
        ApiError, AppState, Log500, handlers::metrics::check_completion::check_deck_completion,
        middleware::AuthenticatedUser,
    },
    outbound::archidekt::{ArchidektClient, ArchidektError},
};

#[cfg(feature = "zerver")]
impl From<ArchidektError> for ApiError {
    fn from(value: ArchidektError) -> Self {
        match value {
            ArchidektError::NotFound => Self::NotFound(
                "deck not found on archidekt, make sure it's public and the url is correct"
                    .to_string(),
            ),
            ArchidektError::Upstream(code) => {
                tracing::warn!(status = code, "archidekt upstream error");
                Self::InternalServerError("failed to fetch deck from archidekt".to_string())
            }
            ArchidektError::Network(e) => e.log_500(),
        }
    }
}

/// Imports an Archidekt deck's cards into an existing deck owned by the
/// authenticated user.
#[cfg(feature = "zerver")]
pub async fn import_archidekt_deck(
    user: AuthenticatedUser,
    Path(deck_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(body): Json<HttpImportArchidektDeck>,
) -> Result<(StatusCode, Json<ImportDeckCardsResult>), ApiError> {
    let archidekt_id = ArchidektClient::extract_deck_id(&body.url).ok_or_else(|| {
        ApiError::UnprocessableEntity(
            "could not parse an archidekt deck id from the url".to_string(),
        )
    })?;
    let board = body
        .board
        .as_deref()
        .map(zwipe_core::domain::deck::Board::try_from)
        .transpose()
        .map_err(|_| ApiError::UnprocessableEntity("invalid board value".to_string()))?
        .unwrap_or_default();

    let db_user = state.user_service.get_user(&GetUser::from(user.id)).await?;
    let email_verified = db_user.email_verified_at.is_some();

    let cards = ArchidektClient::new(&state.web_base_url)
        .fetch_deck(archidekt_id)
        .await?;

    let result = state
        .deck_service
        .import_archidekt_deck(user.id, deck_id, &cards, board, email_verified, body.mode)
        .await
        .map_err(ApiError::from)?;

    // Fire-and-forget metrics: check whether the import completed the deck.
    let deck_service = std::sync::Arc::clone(&state.deck_service);
    let metrics = std::sync::Arc::clone(&state.metrics_service);
    tokio::spawn(check_deck_completion(
        deck_service,
        metrics,
        user.id,
        deck_id,
    ));

    Ok((StatusCode::OK, Json(result)))
}
