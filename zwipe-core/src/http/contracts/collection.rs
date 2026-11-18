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

    /// No export cell reads as `Unknown`: an unread value is not given, never a stored `unknown`.
    #[test]
    fn no_cell_parses_to_unknown() {
        assert!("unknown".parse::<Finish>().is_err());
        assert!("unknown".parse::<CardCondition>().is_err());
        assert!("unknown".parse::<CardLanguage>().is_err());
    }
}
