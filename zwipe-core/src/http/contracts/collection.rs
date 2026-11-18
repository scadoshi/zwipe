//! Collection import: the request carries a whole export, the reply says what landed and which
//! rows didn't.

use serde::{Deserialize, Serialize};

use crate::domain::collection::CollectionSource;
use crate::domain::deck::ImportMode;

/// Imports a ManaBox or Archidekt collection export into the caller's collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpImportCollection {
    /// The export file's text, header row first.
    pub csv: String,
    /// `add` puts the export on top of the collection, summing copies of a printing it already
    /// holds; `replace` makes the collection exactly the export.
    pub mode: ImportMode,
}

/// What a collection import did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpCollectionImportSummary {
    /// Whose export it was.
    pub source: CollectionSource,
    /// Card rows read, matched or not.
    pub rows: u32,
    /// Collection rows the import created.
    pub added: u32,
    /// Collection rows it changed.
    pub updated: u32,
    /// Collection rows it removed: on a replace, those the export no longer holds.
    pub removed: u32,
    /// Copies in the collection afterwards.
    pub cards: i64,
    /// The rows that didn't match a card, the first 200 of them.
    pub unmatched: Vec<UnmatchedRow>,
    /// How many rows didn't match, listed or not.
    pub unmatched_total: u32,
}

/// An export row the import left out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnmatchedRow {
    /// Its line in the file, 1 for the header.
    pub line: u64,
    /// The name the row gave, empty when it gave none.
    pub name: String,
    /// The set code the row gave.
    pub set: Option<String>,
    /// Why it was left out.
    pub reason: UnmatchedReason,
}

/// Why an export row was left out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnmatchedReason {
    /// No printing with its Scryfall id, or its set and collector number.
    UnknownPrinting,
    /// Only a name, and more than one printing has it.
    AmbiguousName,
    /// No name, Scryfall id, or set and collector number to go on.
    MissingName,
    /// A quantity that isn't 1 to 9999.
    BadQuantity,
}
