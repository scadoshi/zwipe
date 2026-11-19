//! Port traits for collection operations.

use std::future::Future;

use uuid::Uuid;
use zwipe_core::domain::collection::{CollectionPageSize, CollectionSearch};
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::http::contracts::collection::{HttpCollectionImportSummary, HttpCollectionSummary};

use crate::domain::collection::models::export_file::ExportLine;
use crate::domain::collection::models::import::{
    CollectionCard, ImportCollection, ImportCollectionFailure, ImportOutcome, ImportRequest,
    Lookups, Wanted,
};
use crate::domain::collection::models::rows::{
    ExportFailure, SetRow, SetRowFailure, SortKey, StoredPage, StoredRow,
};

/// Database port for collections.
pub trait CollectionRepository: Clone + Send + Sync + 'static {
    /// The printings `wanted` can match: by Scryfall id, by set code and collector number (the
    /// English printing first), and by name, within a set and overall, where one printing is a
    /// match. A printing with no oracle id matches nothing.
    fn fetch_printings(
        &self,
        wanted: &Wanted,
    ) -> impl Future<Output = Result<Lookups, anyhow::Error>> + Send;

    /// Writes `cards` into `user_id`'s collection in one transaction, under a lock per user so
    /// two imports run one after the other. `Add` sums copies into a row already there and joins
    /// tags; `Replace` sets each row to the import's and removes every row the import doesn't
    /// hold. With a `request`, an import that used its id in the last day decides first: the
    /// same request answers what that import did and writes nothing, and another mode or file
    /// writes nothing either. A written import records what it did under the request.
    fn apply_import(
        &self,
        user_id: Uuid,
        mode: ImportMode,
        cards: &[CollectionCard],
        request: Option<&ImportRequest>,
    ) -> impl Future<Output = Result<ImportOutcome, anyhow::Error>> + Send;

    /// How many distinct cards and printings `user_id`'s collection holds.
    fn fetch_summary(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<HttpCollectionSummary, anyhow::Error>> + Send;

    /// The printings of the card `oracle_id` in `user_id`'s collection, each once.
    fn fetch_owned_printing_ids(
        &self,
        user_id: Uuid,
        oracle_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Uuid>, anyhow::Error>> + Send;

    /// Gives one row its copies, under the same per-user lock an import takes: inserts or
    /// changes it, or removes it at 0. Returns the row as stored, `None` once removed.
    fn set_row(
        &self,
        row: &SetRow,
    ) -> impl Future<Output = Result<Option<StoredRow>, SetRowFailure>> + Send;

    /// The page of `user_id`'s rows after `after` (from the start without one), `limit` long,
    /// whose card names hold `name` (ignoring case and accents; every row when it is empty), in
    /// [`SortKey`]'s order.
    fn fetch_page(
        &self,
        user_id: Uuid,
        name: &CollectionSearch,
        after: Option<&SortKey>,
        limit: CollectionPageSize,
    ) -> impl Future<Output = Result<StoredPage, anyhow::Error>> + Send;

    /// Every row of `user_id`'s collection with its card's facts, in the list's order, for an
    /// export.
    fn fetch_export_lines(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<ExportLine>, ExportFailure>> + Send;
}

/// Service port for collections.
pub trait CollectionService: Clone + Send + Sync + 'static {
    /// Matches an export's rows to printings and writes them; the reply lists what didn't match.
    fn import_collection(
        &self,
        request: ImportCollection,
    ) -> impl Future<Output = Result<HttpCollectionImportSummary, ImportCollectionFailure>> + Send;

    /// How many distinct cards and printings `user_id`'s collection holds.
    fn fetch_summary(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<HttpCollectionSummary, anyhow::Error>> + Send;

    /// The printings of the card `oracle_id` in `user_id`'s collection, each once.
    fn fetch_owned_printing_ids(
        &self,
        user_id: Uuid,
        oracle_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Uuid>, anyhow::Error>> + Send;

    /// Gives one row its copies; see [`CollectionRepository::set_row`].
    fn set_row(
        &self,
        row: &SetRow,
    ) -> impl Future<Output = Result<Option<StoredRow>, SetRowFailure>> + Send;

    /// One page of `user_id`'s rows; see [`CollectionRepository::fetch_page`].
    fn fetch_page(
        &self,
        user_id: Uuid,
        name: &CollectionSearch,
        after: Option<&SortKey>,
        limit: CollectionPageSize,
    ) -> impl Future<Output = Result<StoredPage, anyhow::Error>> + Send;

    /// `user_id`'s collection as a ManaBox export's text.
    fn export(&self, user_id: Uuid) -> impl Future<Output = Result<String, ExportFailure>> + Send;
}
