//! The collection's stored rows as the edit and list calls read and write them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionCursor, CollectionQuantity, CollectionTags, Finish,
    SetQuantity,
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
    /// The last row's place in the list when more rows follow it.
    pub next: Option<SortKey>,
}

/// A row's place in the collection list's order: card name, set code, the collector number's
/// leading digits as a number (`Infinity` without any), the whole collector number, finish,
/// condition and language (empty when not given), then printing. A cursor carries the last row's
/// key, and the next page is the rows after it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortKey {
    /// The card's name.
    pub card_name: String,
    /// The printing's set code.
    pub set_code: String,
    /// The collector number's leading digits, or `Infinity`, as Postgres writes the number.
    pub number: String,
    /// The collector number.
    pub collector_number: String,
    /// The finish's stored spelling.
    pub finish: String,
    /// The condition's stored spelling, empty when not given.
    pub condition: String,
    /// The language's stored spelling, empty when not given.
    pub language: String,
    /// The printing.
    pub scryfall_data_id: Uuid,
}

impl SortKey {
    /// The cursor that carries this key: its JSON in hex.
    #[must_use]
    pub fn cursor(&self) -> Option<CollectionCursor> {
        let json = serde_json::to_vec(self).ok()?;
        CollectionCursor::new(&hex::encode(json)).ok()
    }

    /// The key a cursor carries, or `None` for a cursor this server didn't write.
    #[must_use]
    pub fn of_cursor(cursor: &CollectionCursor) -> Option<Self> {
        let json = hex::decode(cursor.as_str()).ok()?;
        serde_json::from_slice(&json).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_round_trips_through_its_cursor() {
        let key = SortKey {
            card_name: "Lim-Dûl's \"Vault\"".to_owned(),
            set_code: "all".to_owned(),
            number: "Infinity".to_owned(),
            collector_number: "★12".to_owned(),
            finish: "foil".to_owned(),
            condition: String::new(),
            language: "ja".to_owned(),
            scryfall_data_id: Uuid::from_u128(7),
        };
        let cursor = key.cursor().unwrap();
        assert_eq!(SortKey::of_cursor(&cursor), Some(key));
        assert_eq!(
            SortKey::of_cursor(&CollectionCursor::new("0a1b").unwrap()),
            None
        );
    }
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

/// Why an export wrote nothing.
#[derive(Debug, thiserror::Error)]
pub enum ExportFailure {
    /// The collection holds more rows than an import takes back.
    #[error("the collection holds more than {max} rows, more than an import takes back", max = zwipe_core::domain::collection::limits::MAX_ROWS)]
    TooManyRows,
    /// The database failed.
    #[error(transparent)]
    Database(#[from] anyhow::Error),
}
