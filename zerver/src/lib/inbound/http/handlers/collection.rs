//! Collection import, the collection's reads for search, and editing and listing its rows.

use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionCursor, CollectionPageSize, CollectionSearch, Finish,
    ImportCollectionError,
};
use zwipe_core::http::contracts::collection::{
    CollectionCard, HttpCollectionExport, HttpCollectionImportSummary, HttpCollectionPage,
    HttpCollectionSummary, HttpImportCollection, HttpSetCollectionCard, HttpSetCollectionCardReply,
};

use crate::domain::card::ports::CardService;
use crate::domain::card::requests::get_scryfall_data::ScryfallDataIds;
use crate::domain::collection::models::export;
use crate::domain::collection::models::import::{ImportCollection, ImportCollectionFailure};
use crate::domain::collection::models::rows::{
    ExportFailure, SetRow, SetRowFailure, SortKey, StoredRow,
};
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

impl From<SetRowFailure> for ApiError {
    fn from(value: SetRowFailure) -> Self {
        match value {
            SetRowFailure::UnknownPrinting => Self::UnprocessableEntity(value.to_string()),
            SetRowFailure::Database(e) => e.to_500(),
        }
    }
}

/// Each of `rows` with its card, in order, read in one batch. A row whose card the catalog no
/// longer serves is left out.
async fn with_cards(
    state: &AppState,
    rows: Vec<StoredRow>,
) -> Result<Vec<CollectionCard>, ApiError> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let ids: ScryfallDataIds = rows.iter().map(|row| row.scryfall_data_id).collect();
    let cards: HashMap<Uuid, Card> = state
        .card_service
        .fetch_cards(&ids)
        .await
        .map_err(ApiError::from)?
        .into_iter()
        .map(|card| (card.scryfall_data.id, card))
        .collect();
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let card = cards.get(&row.scryfall_data_id)?.clone();
            Some(CollectionCard {
                card,
                finish: row.finish,
                condition: row.condition,
                language: row.language,
                quantity: row.quantity,
                tags: row.tags,
                added_at: row.added_at,
            })
        })
        .collect())
}

/// The line a set call gets for a value this server reads as unknown.
fn unknown_value(field: &str) -> ApiError {
    ApiError::UnprocessableEntity(format!("unknown {field}"))
}

/// Gives one row of the caller's collection its copies: the row with the call's printing,
/// finish, condition and language (left out, the manual-add row), 0 to remove it. Answers with
/// the row as it is now, or none once removed.
///
/// # Errors
///
/// Returns 422 for a printing the collection doesn't take, a finish, condition or language this
/// server doesn't know, or copies outside 0 to 9999; 401 without a session; 429 past the rate
/// limit; and 500 on a database failure.
pub async fn set_collection_card(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<HttpSetCollectionCard>,
) -> Result<(StatusCode, Json<HttpSetCollectionCardReply>), ApiError> {
    let HttpSetCollectionCard {
        scryfall_data_id,
        finish,
        condition,
        language,
        quantity,
    } = body;
    if finish == Finish::Unknown {
        return Err(unknown_value("finish"));
    }
    if condition == Some(CardCondition::Unknown) {
        return Err(unknown_value("condition"));
    }
    if language == Some(CardLanguage::Unknown) {
        return Err(unknown_value("language"));
    }
    let row = SetRow {
        user_id: user.id,
        scryfall_data_id,
        finish,
        condition,
        language,
        quantity,
    };
    let stored = state.collection_service.set_row(&row).await?;
    let card = with_cards(&state, stored.into_iter().collect())
        .await?
        .into_iter()
        .next();
    Ok((StatusCode::OK, Json(HttpSetCollectionCardReply { card })))
}

/// A list call's query as the URL gives it, checked by [`collection_page`].
#[derive(Debug, Deserialize)]
pub struct PageQuery {
    name: Option<String>,
    after: Option<String>,
    limit: Option<String>,
}

/// One page of the caller's collection, sorted by card name, then set, collector number and
/// finish, searchable by card name ignoring case and accents. A page starts after the cursor a
/// reply's `next` gave, so rows added or removed before it never move what follows.
///
/// # Errors
///
/// Returns 422 for a search past 100 characters, a cursor this server didn't write, or a page
/// size outside 1 to 100; 401 without a session; and 500 on a database failure.
pub async fn collection_page(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> Result<(StatusCode, Json<HttpCollectionPage>), ApiError> {
    let unprocessable = |e: &dyn std::fmt::Display| ApiError::UnprocessableEntity(e.to_string());
    let name = CollectionSearch::new(query.name.as_deref().unwrap_or_default())
        .map_err(|e| unprocessable(&e))?;
    let after = match query.after.as_deref() {
        None | Some("") => None,
        Some(text) => {
            let cursor = CollectionCursor::new(text).map_err(|e| unprocessable(&e))?;
            let key = SortKey::of_cursor(&cursor).ok_or_else(|| {
                ApiError::UnprocessableEntity("not a collection cursor".to_owned())
            })?;
            Some(key)
        }
    };
    let limit = match query.limit.as_deref() {
        None | Some("") => CollectionPageSize::default(),
        Some(text) => {
            let rows: u32 = text
                .parse()
                .map_err(|_| ApiError::UnprocessableEntity("limit is a whole number".to_owned()))?;
            CollectionPageSize::new(rows).map_err(|e| unprocessable(&e))?
        }
    };
    let page = state
        .collection_service
        .fetch_page(user.id, &name, after.as_ref(), limit)
        .await
        .map_err(|e| e.to_500())?;
    let next = page.next.as_ref().and_then(SortKey::cursor);
    let cards = with_cards(&state, page.rows).await?;
    Ok((
        StatusCode::OK,
        Json(HttpCollectionPage {
            cards,
            total: page.total,
            next,
        }),
    ))
}

impl From<ExportFailure> for ApiError {
    fn from(value: ExportFailure) -> Self {
        match value {
            ExportFailure::TooManyRows => Self::UnprocessableEntity(value.to_string()),
            ExportFailure::Database(e) => e.to_500(),
        }
    }
}

/// The caller's collection as a ManaBox export, which a collection import reads back, named for
/// today's date.
///
/// # Errors
///
/// Returns 422 for a collection with more rows than an import takes back, 401 without a
/// session, 429 past the rate limit it shares with the import, and 500 on a database failure.
pub async fn export_collection(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<HttpCollectionExport>), ApiError> {
    let csv = state.collection_service.export(user.id).await?;
    let file_name = format!(
        "zwipe-collection-{}.csv",
        chrono::Utc::now().format("%Y-%m-%d")
    );
    Ok((
        StatusCode::OK,
        Json(HttpCollectionExport { file_name, csv }),
    ))
}
