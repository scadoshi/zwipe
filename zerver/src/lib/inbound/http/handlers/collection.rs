//! Collection import.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::collection::ImportCollectionError;
use zwipe_core::http::contracts::collection::{HttpCollectionImportSummary, HttpImportCollection};

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
