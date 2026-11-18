//! The collection service: matching an export's rows and writing them.

use zwipe_core::domain::collection::ImportCollectionError;
use zwipe_core::http::contracts::collection::HttpCollectionImportSummary;

use crate::domain::collection::models::import::{
    self, ImportCollection, ImportCollectionFailure, Wanted,
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
        } = request;
        let rows_read = rows.len();
        let lookups = self.repo.fetch_printings(&Wanted::of(&rows)).await?;
        let resolved = import::resolve(rows, &lookups);
        if resolved.cards.is_empty() {
            return Err(ImportCollectionError::NothingMatched.into());
        }
        let applied = self
            .repo
            .apply_import(user_id, mode, &resolved.cards)
            .await?;
        Ok(import::summary(source, rows_read, resolved, applied))
    }
}
