//! Collection import, and the collection's reads for search.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::collection::ImportCollectionError;
use zwipe_core::http::contracts::collection::{
    HttpCollectionImportSummary, HttpCollectionSummary, HttpImportCollection,
};

use crate::domain::card::ports::CardService;
use crate::domain::card::requests::get_scryfall_data::ScryfallDataIds;
use crate::domain::collection::models::export;
use crate::domain::collection::models::import::{ImportCollection, ImportCollectionFailure};
use crate::domain::collection::ports::CollectionService;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<ImportCollectionError> for ApiError {
    fn from(value: ImportCollectionError) -> Self {
        Self::UnprocessableEntity(value.to_string())
    }
}

impl From<ImportCollectionFailure> for ApiError {
    fn from(value: ImportCollectionFailure) -> Self {
        match value {
            ImportCollectionFailure::Invalid(e) => Self::from(e),
            ImportCollectionFailure::Database(e) => e.to_500(),
        }
    }
}

/// Imports a ManaBox or Archidekt collection export into the caller's collection. The file is
/// read off the async runtime, and the write is one transaction.
///
/// # Errors
///
/// Returns 413 for a body over [`MAX_BODY_BYTES`], 422 for a file the import can't take (with
/// a message that says why), 429 past the per-account or per-address rate limit, 401 without a
/// session, and 500 on a database failure. Every error leaves the collection as it was.
///
/// [`MAX_BODY_BYTES`]: zwipe_core::domain::collection::limits::MAX_BODY_BYTES
pub async fn import_collection(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<HttpImportCollection>,
) -> Result<(StatusCode, Json<HttpCollectionImportSummary>), ApiError> {
    let HttpImportCollection { csv, mode } = body;
    let read = tokio::task::spawn_blocking(move || export::read(&csv))
        .await
        .map_err(To500::to_500)??;
    let request = ImportCollection {
        user_id: user.id,
        mode,
        source: read.source,
        rows: read.rows,
    };
    let summary = state.collection_service.import_collection(request).await?;
    Ok((StatusCode::OK, Json(summary)))
}

/// How many distinct cards and printings the caller's collection holds.
///
/// # Errors
///
/// Returns 401 without a session and 500 on a database failure.
pub async fn collection_summary(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<HttpCollectionSummary>), ApiError> {
    let summary = state
        .collection_service
        .fetch_summary(user.id)
        .await
        .map_err(|e| e.to_500())?;
    Ok((StatusCode::OK, Json(summary)))
}

/// The printings of a card, by oracle id, that the caller's collection holds, each once, oldest
/// release first. They are read by id, so the list holds every owned printing the card catalog
/// can serve (every synced printing, English or not) whatever the public printings route
/// shows. A card the caller doesn't own has none.
///
/// # Errors
///
/// Returns 422 for a malformed oracle id, 401 without a session, and 500 on a database failure.
pub async fn owned_printings(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Path(oracle_id): Path<String>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    let oracle_id = uuid::Uuid::try_parse(&oracle_id)
        .map_err(|e| ApiError::UnprocessableEntity(format!("invalid oracle id: {e}")))?;
    let owned = state
        .collection_service
        .fetch_owned_printing_ids(user.id, oracle_id)
        .await
        .map_err(|e| e.to_500())?;
    if owned.is_empty() {
        return Ok((StatusCode::OK, Json(Vec::new())));
    }
    let ids: ScryfallDataIds = owned.into_iter().collect();
    let mut mine = state
        .card_service
        .fetch_cards(&ids)
        .await
        .map_err(ApiError::from)?;
    mine.sort_by_key(|card| (card.scryfall_data.released_at, card.scryfall_data.id));
    Ok((StatusCode::OK, Json(mine)))
}
