//! The collection's stored rows as the edit and list calls read and write them.

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionQuantity, CollectionTags, Finish, SetQuantity,
};

/// A set call: give the row of `user_id`'s collection with this printing, finish, condition and
/// language `quantity` copies, removing it at 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetRow {
    /// Whose collection.
    pub user_id: Uuid,
    /// The printing.
    pub scryfall_data_id: Uuid,
    /// The row's finish, never [`Finish::Unknown`].
    pub finish: Finish,
    /// The row's condition, never [`CardCondition::Unknown`].
    pub condition: Option<CardCondition>,
    /// The row's language, never [`CardLanguage::Unknown`].
    pub language: Option<CardLanguage>,
    /// The row's copies from now on.
    pub quantity: SetQuantity,
}

/// One stored row of a collection, without its card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredRow {
    /// The printing.
    pub scryfall_data_id: Uuid,
    /// Its finish.
    pub finish: Finish,
    /// Its condition, `None` when not given.
    pub condition: Option<CardCondition>,
    /// Its language, `None` when not given.
    pub language: Option<CardLanguage>,
    /// Its copies.
    pub quantity: CollectionQuantity,
    /// Its tags.
    pub tags: CollectionTags,
    /// When the export says it was added.
    pub added_at: Option<DateTime<Utc>>,
}

/// One page of a collection's rows and how many rows the search matches in all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredPage {
    /// The page's rows, in the list's order.
    pub rows: Vec<StoredRow>,
    /// Rows the search matches across every page.
    pub total: i64,
}

/// Why a set call stored nothing.
#[derive(Debug, thiserror::Error)]
pub enum SetRowFailure {
    /// The printing isn't one the collection takes: no such printing, or one with no card.
    #[error("no card with that printing")]
    UnknownPrinting,
    /// The database failed.
    #[error(transparent)]
    Database(#[from] anyhow::Error),
}
