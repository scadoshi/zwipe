//! The collection's calls: an import carries a whole export and says what landed and which rows
//! didn't; a set call gives one row its copies; the list reads the collection a page at a time;
//! the export hands it back as a ManaBox file.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::card::Card;
use crate::domain::collection::{
    CardCondition, CardLanguage, CollectionCursor, CollectionQuantity, CollectionSource,
    CollectionTags, Finish, SetQuantity,
};
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

/// What the caller's collection holds. A client offers "In my collection" only when it holds
/// something, and only a server with the owned filter answers this route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpCollectionSummary {
    /// Distinct cards, by oracle id, with at least one printing in the collection.
    pub cards: i64,
    /// Distinct printings in the collection, whatever their finish, condition or language.
    pub printings: i64,
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
    /// The rows that didn't match a card, the first [`MAX_UNMATCHED_LISTED`](crate::domain::collection::limits::MAX_UNMATCHED_LISTED) of them.
    pub unmatched: Vec<UnmatchedRow>,
    /// How many rows didn't match, listed or not.
    pub unmatched_total: u32,
}

/// An export row the import left out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnmatchedRow {
    /// Its line in the file, 1 for the header.
    pub line: u64,
    /// The name the row gave.
    pub name: Option<String>,
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
    /// A quantity outside 1 to [`CollectionQuantity::MAX`](crate::domain::collection::CollectionQuantity::MAX).
    BadQuantity,
    /// A value this build doesn't know, from a newer server. The server never stores or sends
    /// it for a value it knows.
    #[serde(other)]
    Unknown,
}

/// One row of the caller's collection: a printing in one finish, condition and language, and how
/// many copies of it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectionCard {
    /// The printing, as search serves it.
    pub card: Card,
    /// Its finish.
    pub finish: Finish,
    /// Its condition, `None` when not given.
    pub condition: Option<CardCondition>,
    /// Its language, `None` when not given.
    pub language: Option<CardLanguage>,
    /// Its copies.
    pub quantity: CollectionQuantity,
    /// Its tags, from an Archidekt export; empty for every other row.
    pub tags: CollectionTags,
    /// When the export says it was added, `None` without one.
    pub added_at: Option<DateTime<Utc>>,
}

/// Gives one row of the caller's collection its copies: the row with this printing, finish,
/// condition and language. A condition or language left out is "not given", the row a manual
/// add makes; with them the call edits exactly the row the list showed. 0 copies removes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpSetCollectionCard {
    /// The printing's Scryfall id.
    pub scryfall_data_id: uuid::Uuid,
    /// The row's finish.
    pub finish: Finish,
    /// The row's condition.
    #[serde(default)]
    pub condition: Option<CardCondition>,
    /// The row's language.
    #[serde(default)]
    pub language: Option<CardLanguage>,
    /// The row's copies from now on, 0 to remove it.
    pub quantity: SetQuantity,
}

/// What a set call left: the row, or `None` when it removed it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpSetCollectionCardReply {
    /// The row as it is now.
    pub card: Option<CollectionCard>,
}

/// One page of the caller's collection, sorted by card name, then set, collector number and
/// finish.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpCollectionPage {
    /// The page's rows.
    pub cards: Vec<CollectionCard>,
    /// Rows the search matches across every page.
    pub total: i64,
    /// Where the next page starts, handed back as `after`; `None` on the last page.
    pub next: Option<CollectionCursor>,
}

/// The caller's collection as a ManaBox export: its header row and one line per row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpCollectionExport {
    /// A file name for the download, ending `.csv`.
    pub file_name: String,
    /// The file's text.
    pub csv: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::collection::{CardCondition, CardLanguage, Finish};

    /// A value a newer server sends reads as `Unknown` in an older client, and the reply still
    /// parses.
    #[test]
    fn a_reply_enum_value_this_build_doesn_t_know_reads_as_unknown() {
        let newer = "\"from_a_newer_server\"";
        assert_eq!(
            serde_json::from_str::<UnmatchedReason>(newer).unwrap(),
            UnmatchedReason::Unknown
        );
        assert_eq!(
            serde_json::from_str::<CollectionSource>(newer).unwrap(),
            CollectionSource::Unknown
        );
        assert_eq!(
            serde_json::from_str::<Finish>(newer).unwrap(),
            Finish::Unknown
        );
        assert_eq!(
            serde_json::from_str::<CardCondition>(newer).unwrap(),
            CardCondition::Unknown
        );
        assert_eq!(
            serde_json::from_str::<CardLanguage>(newer).unwrap(),
            CardLanguage::Unknown
        );
        let reply = r#"{"source":"moxfield","rows":1,"added":0,"updated":0,"removed":0,"cards":0,
            "unmatched":[{"line":2,"name":null,"set":null,"reason":"too_spicy"}],"unmatched_total":1}"#;
        let summary: HttpCollectionImportSummary = serde_json::from_str(reply).unwrap();
        assert_eq!(summary.source, CollectionSource::Unknown);
        assert_eq!(
            summary.unmatched.first().map(|row| row.reason),
            Some(UnmatchedReason::Unknown)
        );
    }

    /// A set call that leaves out condition and language names the manual-add row.
    #[test]
    fn a_set_call_without_condition_or_language_names_the_manual_row() {
        let body = r#"{"scryfall_data_id":"00000000-0000-0000-0000-000000000001","finish":"foil","quantity":0}"#;
        let call: HttpSetCollectionCard = serde_json::from_str(body).unwrap();
        assert_eq!(call.condition, None);
        assert_eq!(call.language, None);
        assert_eq!(call.quantity.row(), None, "0 removes");
        let body = r#"{"scryfall_data_id":"00000000-0000-0000-0000-000000000001","finish":"nonfoil","condition":"near_mint","language":"ja","quantity":10000}"#;
        assert!(
            serde_json::from_str::<HttpSetCollectionCard>(body).is_err(),
            "past the most"
        );
    }

    /// No export cell reads as `Unknown`: an unread value is not given, never a stored `unknown`.
    #[test]
    fn no_cell_parses_to_unknown() {
        assert!("unknown".parse::<Finish>().is_err());
        assert!("unknown".parse::<CardCondition>().is_err());
        assert!("unknown".parse::<CardLanguage>().is_err());
    }
}
