//! Port traits for collection operations.

use std::future::Future;

use uuid::Uuid;
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::http::contracts::collection::HttpCollectionImportSummary;

use crate::domain::collection::models::import::{
    Applied, CollectionCard, ImportCollection, ImportCollectionFailure, Lookups, Wanted,
};

/// Database port for collections.
pub trait CollectionRepository: Clone + Send + Sync + 'static {
    /// The printings `wanted` can match: by Scryfall id, by set code and collector number (the
    /// English printing first), and how many printings carry each name, within a set and
    /// overall. A printing with no oracle id matches nothing.
    fn fetch_printings(
        &self,
        wanted: &Wanted,
    ) -> impl Future<Output = Result<Lookups, anyhow::Error>> + Send;

    /// Writes `cards` into `user_id`'s collection in one transaction, the user's row locked so
    /// two imports run one after the other. `Add` sums copies into a row already there and joins
    /// tags; `Replace` sets each row to the import's and removes every row the import doesn't
    /// hold.
    fn apply_import(
        &self,
        user_id: Uuid,
        mode: ImportMode,
        cards: &[CollectionCard],
    ) -> impl Future<Output = Result<Applied, anyhow::Error>> + Send;
}

/// Service port for collections.
pub trait CollectionService: Clone + Send + Sync + 'static {
    /// Matches an export's rows to printings and writes them; the reply lists what didn't match.
    fn import_collection(
        &self,
        request: ImportCollection,
    ) -> impl Future<Output = Result<HttpCollectionImportSummary, ImportCollectionFailure>> + Send;
}
