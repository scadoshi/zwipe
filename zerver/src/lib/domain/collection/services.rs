//! The collection service: matching an export's rows and writing them.

use uuid::Uuid;
use zwipe_core::domain::collection::{CollectionPageSize, CollectionSearch, ImportCollectionError};
use zwipe_core::http::contracts::collection::{HttpCollectionImportSummary, HttpCollectionSummary};

use crate::domain::collection::models::export_file;
use crate::domain::collection::models::import::{
    self, ImportCollection, ImportCollectionFailure, ImportOutcome, Wanted,
};
use crate::domain::collection::models::rows::{
    ExportFailure, SetRow, SetRowFailure, SortKey, StoredPage, StoredRow,
};
use crate::domain::collection::ports::{CollectionRepository, CollectionService};

/// Collection operations over a [`CollectionRepository`].
#[derive(Debug, Clone)]
pub struct Service<R> {
    repo: R,
}

impl<R: CollectionRepository> Service<R> {
    /// A service over `repo`.
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: CollectionRepository> CollectionService for Service<R> {
    async fn import_collection(
        &self,
        request: ImportCollection,
    ) -> Result<HttpCollectionImportSummary, ImportCollectionFailure> {
        let ImportCollection {
            user_id,
            mode,
            source,
            rows,
            request,
        } = request;
        let rows_read = rows.len();
        let lookups = self.repo.fetch_printings(&Wanted::of(&rows)).await?;
        let resolved = import::resolve(rows, &lookups);
        if resolved.cards.is_empty() {
            return Err(ImportCollectionError::NothingMatched.into());
        }
        let applied = match self
            .repo
            .apply_import(user_id, mode, &resolved.cards, request.as_ref())
            .await?
        {
            ImportOutcome::Applied(applied) | ImportOutcome::Repeated(applied) => applied,
            ImportOutcome::IdReused => return Err(ImportCollectionError::RequestReused.into()),
        };
        Ok(import::summary(source, rows_read, resolved, applied))
    }

    async fn fetch_summary(&self, user_id: Uuid) -> Result<HttpCollectionSummary, anyhow::Error> {
        self.repo.fetch_summary(user_id).await
    }

    async fn fetch_owned_printing_ids(
        &self,
        user_id: Uuid,
        oracle_id: Uuid,
    ) -> Result<Vec<Uuid>, anyhow::Error> {
        self.repo.fetch_owned_printing_ids(user_id, oracle_id).await
    }

    async fn set_row(&self, row: &SetRow) -> Result<Option<StoredRow>, SetRowFailure> {
        self.repo.set_row(row).await
    }

    async fn fetch_page(
        &self,
        user_id: Uuid,
        name: &CollectionSearch,
        after: Option<&SortKey>,
        limit: CollectionPageSize,
    ) -> Result<StoredPage, anyhow::Error> {
        self.repo.fetch_page(user_id, name, after, limit).await
    }

    async fn export(&self, user_id: Uuid) -> Result<String, ExportFailure> {
        let lines = self.repo.fetch_export_lines(user_id).await?;
        tokio::task::spawn_blocking(move || export_file::write(&lines))
            .await
            .map_err(anyhow::Error::from)?
            .map_err(ExportFailure::Database)
    }
}
