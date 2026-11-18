//! How an export's rows find their printings, merge, and what the import reports. Each row tries,
//! first hit wins: its Scryfall id; its set code and collector number; its name within its set;
//! its name alone, only when one printing has it. A row nothing matches is reported, never
//! guessed. Rows that land on the same printing, finish, condition and language merge.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zwipe_core::domain::collection::limits::MAX_UNMATCHED_LISTED;
use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionQuantity, CollectionSource, CollectionTags, Finish,
    ImportCollectionError, PurchaseCurrency, PurchasePrice,
};
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::http::contracts::collection::{
    HttpCollectionImportSummary, UnmatchedReason, UnmatchedRow,
};

use crate::domain::collection::models::export::ExportRow;

/// A printing a row matched: its id and its card's oracle id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Printing {
    /// The printing's id (`scryfall_data.id`).
    pub id: Uuid,
    /// Its card's oracle id.
    pub oracle_id: Uuid,
}

/// What a name, or a name within a set, matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameMatch {
    /// Exactly one printing.
    One(Printing),
    /// More than one, so the name alone can't say which.
    Many,
}

/// What the rows asked the database for, each list without repeats. Names are lowercased.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Wanted {
    /// Scryfall ids.
    pub ids: Vec<Uuid>,
    /// Set codes and collector numbers.
    pub set_numbers: Vec<(String, String)>,
    /// Names within a set.
    pub name_sets: Vec<(String, String)>,
    /// Names.
    pub names: Vec<String>,
}

impl Wanted {
    /// Everything `rows` could match by. A row with a quantity outside the range asks for
    /// nothing, since it won't import.
    #[must_use]
    pub fn of(rows: &[ExportRow]) -> Self {
        let mut wanted = Self::default();
        for row in rows.iter().filter(|row| row.quantity.is_some()) {
            if let Some(id) = row.scryfall_id {
                wanted.ids.push(id);
            }
            if let (Some(set), Some(number)) = (&row.set_code, &row.collector_number) {
                wanted.set_numbers.push((set.clone(), number.clone()));
            }
            if let Some(name) = &row.name {
                let name = name.to_lowercase();
                if let Some(set) = &row.set_code {
                    wanted.name_sets.push((name.clone(), set.clone()));
                }
                wanted.names.push(name);
            }
        }
        dedupe(&mut wanted.ids);
        dedupe(&mut wanted.set_numbers);
        dedupe(&mut wanted.name_sets);
        dedupe(&mut wanted.names);
        wanted
    }
}

fn dedupe<T: Ord>(list: &mut Vec<T>) {
    list.sort();
    list.dedup();
}

/// What the database found for a [`Wanted`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lookups {
    /// By Scryfall id.
    pub by_id: HashMap<Uuid, Printing>,
    /// By set code and collector number.
    pub by_set_number: HashMap<(String, String), Printing>,
    /// By lowercased name within a set.
    pub by_name_set: HashMap<(String, String), NameMatch>,
    /// By lowercased name.
    pub by_name: HashMap<String, NameMatch>,
}

/// One collection row to write: a printing in one finish, condition and language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionCard {
    /// The printing.
    pub printing: Printing,
    /// Its finish.
    pub finish: Finish,
    /// Its condition.
    pub condition: Option<CardCondition>,
    /// Its language.
    pub language: Option<CardLanguage>,
    /// The copies.
    pub quantity: CollectionQuantity,
    /// Its tags.
    pub tags: CollectionTags,
    /// What it cost.
    pub purchase_price: Option<PurchasePrice>,
    /// The price's currency.
    pub purchase_currency: Option<PurchaseCurrency>,
    /// When it was added in the app it came from.
    pub added_at: Option<DateTime<Utc>>,
}

/// The key a collection row is unique by.
type CardKey = (Uuid, Finish, Option<CardCondition>, Option<CardLanguage>);

impl CollectionCard {
    fn key(&self) -> CardKey {
        (self.printing.id, self.finish, self.condition, self.language)
    }

    /// Folds `other`, a row with the same key, into this one: copies add up (held at the most a
    /// row holds), tags join (past the cap the later ones are left off), the earliest date added
    /// stays, and the first price given stays.
    fn merge(&mut self, other: Self) {
        self.quantity = self.quantity.merged(other.quantity);
        let mut tags = self.tags.clone();
        for tag in other.tags.as_slice() {
            if tags.add(tag.clone()).is_err() {
                break;
            }
        }
        self.tags = tags;
        self.added_at = match (self.added_at, other.added_at) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        if self.purchase_price.is_none() {
            self.purchase_price = other.purchase_price;
            self.purchase_currency = other.purchase_currency;
        }
    }
}

/// The rows matched and merged, and those left out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolved {
    /// One per key, in the order first seen.
    pub cards: Vec<CollectionCard>,
    /// The rows left out, the first [`MAX_UNMATCHED_LISTED`] of them.
    pub unmatched: Vec<UnmatchedRow>,
    /// How many rows were left out.
    pub unmatched_total: u32,
}

impl Resolved {
    /// Reports `row` as left out for `reason`.
    fn leave_out(&mut self, row: ExportRow, reason: UnmatchedReason) {
        self.unmatched_total = self.unmatched_total.saturating_add(1);
        if self.unmatched.len() < MAX_UNMATCHED_LISTED {
            self.unmatched.push(UnmatchedRow {
                line: row.line,
                name: row.name,
                set: row.set_code,
                reason,
            });
        }
    }
}

/// Matches each of `rows` against `lookups` and merges rows that land on one key.
#[must_use]
pub fn resolve(rows: Vec<ExportRow>, lookups: &Lookups) -> Resolved {
    let mut resolved = Resolved::default();
    let mut index: HashMap<CardKey, usize> = HashMap::new();
    for row in rows {
        let found = match row.quantity {
            None => Err(UnmatchedReason::BadQuantity),
            Some(quantity) => matched(&row, lookups).map(|printing| (printing, quantity)),
        };
        let (printing, quantity) = match found {
            Ok(found) => found,
            Err(reason) => {
                resolved.leave_out(row, reason);
                continue;
            }
        };
        let card = CollectionCard {
            printing,
            finish: row.finish,
            condition: row.condition,
            language: row.language,
            quantity,
            tags: row.tags,
            purchase_price: row.purchase_price,
            purchase_currency: row.purchase_currency,
            added_at: row.added_at,
        };
        let key = card.key();
        if let Some(held) = index.get(&key).and_then(|at| resolved.cards.get_mut(*at)) {
            held.merge(card);
        } else {
            index.insert(key, resolved.cards.len());
            resolved.cards.push(card);
        }
    }
    resolved
}

/// The printing `row` matches, or why none does.
fn matched(row: &ExportRow, lookups: &Lookups) -> Result<Printing, UnmatchedReason> {
    if let Some(found) = row.scryfall_id.and_then(|id| lookups.by_id.get(&id)) {
        return Ok(*found);
    }
    if let (Some(set), Some(number)) = (&row.set_code, &row.collector_number)
        && let Some(found) = lookups.by_set_number.get(&(set.clone(), number.clone()))
    {
        return Ok(*found);
    }
    let Some(name) = row.name.as_ref().map(|name| name.to_lowercase()) else {
        return Err(if row.scryfall_id.is_some() || row.set_code.is_some() {
            UnmatchedReason::UnknownPrinting
        } else {
            UnmatchedReason::MissingName
        });
    };
    if let Some(set) = &row.set_code
        && let Some(NameMatch::One(found)) = lookups.by_name_set.get(&(name.clone(), set.clone()))
    {
        return Ok(*found);
    }
    match lookups.by_name.get(&name) {
        Some(NameMatch::One(found)) => Ok(*found),
        Some(NameMatch::Many) => Err(UnmatchedReason::AmbiguousName),
        None => Err(UnmatchedReason::UnknownPrinting),
    }
}

/// What writing the rows changed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Applied {
    /// Rows created.
    pub added: u32,
    /// Rows changed.
    pub updated: u32,
    /// Rows removed.
    pub removed: u32,
    /// Copies in the collection afterwards.
    pub cards: i64,
}

/// One import: who, how, and the export it read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportCollection {
    /// The collection's owner.
    pub user_id: Uuid,
    /// Add on top, or replace.
    pub mode: ImportMode,
    /// Whose export it was.
    pub source: CollectionSource,
    /// The card rows read.
    pub rows: Vec<ExportRow>,
}

/// An import that changed nothing.
#[derive(Debug, thiserror::Error)]
pub enum ImportCollectionFailure {
    /// A file the import can't take.
    #[error(transparent)]
    Invalid(#[from] ImportCollectionError),
    /// The database failed; the transaction rolled back.
    #[error(transparent)]
    Database(#[from] anyhow::Error),
}

/// The reply for an import of `rows_read` rows.
#[must_use]
pub fn summary(
    source: CollectionSource,
    rows_read: usize,
    resolved: Resolved,
    applied: Applied,
) -> HttpCollectionImportSummary {
    HttpCollectionImportSummary {
        source,
        rows: u32::try_from(rows_read).unwrap_or(u32::MAX),
        added: applied.added,
        updated: applied.updated,
        removed: applied.removed,
        cards: applied.cards,
        unmatched: resolved.unmatched,
        unmatched_total: resolved.unmatched_total,
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::collection::CollectionTag;

    use super::*;

    fn printing(n: u128) -> Printing {
        Printing {
            id: Uuid::from_u128(n),
            oracle_id: Uuid::from_u128(n | 0x1000_0000),
        }
    }

    fn row(line: u64) -> ExportRow {
        ExportRow {
            line,
            name: None,
            scryfall_id: None,
            set_code: None,
            collector_number: None,
            quantity: CollectionQuantity::new(1).ok(),
            finish: Finish::Nonfoil,
            condition: None,
            language: None,
            tags: CollectionTags::default(),
            purchase_price: None,
            purchase_currency: None,
            added_at: None,
        }
    }

    fn lookups() -> Lookups {
        let mut l = Lookups::default();
        l.by_id.insert(Uuid::from_u128(1), printing(1));
        l.by_set_number
            .insert(("c21".into(), "263".into()), printing(2));
        l.by_name_set
            .insert(("forest".into(), "c21".into()), NameMatch::One(printing(3)));
        l.by_name.insert("forest".into(), NameMatch::Many);
        l.by_name
            .insert("lonely".into(), NameMatch::One(printing(4)));
        l
    }

    #[test]
    fn each_row_takes_the_first_way_that_matches() {
        let by_id = ExportRow {
            scryfall_id: Some(Uuid::from_u128(1)),
            set_code: Some("c21".into()),
            collector_number: Some("263".into()),
            ..row(2)
        };
        let by_number = ExportRow {
            scryfall_id: Some(Uuid::from_u128(99)),
            set_code: Some("c21".into()),
            collector_number: Some("263".into()),
            ..row(3)
        };
        let by_name_set = ExportRow {
            name: Some("Forest".into()),
            set_code: Some("c21".into()),
            ..row(4)
        };
        let by_name = ExportRow {
            name: Some("Lonely".into()),
            ..row(5)
        };
        let resolved = resolve(vec![by_id, by_number, by_name_set, by_name], &lookups());
        let got: Vec<Uuid> = resolved.cards.iter().map(|c| c.printing.id).collect();
        assert_eq!(got, [1, 2, 3, 4].map(Uuid::from_u128));
        assert_eq!(resolved.unmatched_total, 0);
    }

    #[test]
    fn a_row_nothing_matches_says_why_and_is_never_guessed() {
        let ambiguous = ExportRow {
            name: Some("Forest".into()),
            ..row(2)
        };
        let unknown = ExportRow {
            scryfall_id: Some(Uuid::from_u128(99)),
            ..row(3)
        };
        let nameless = row(4);
        let bad = ExportRow {
            quantity: None,
            scryfall_id: Some(Uuid::from_u128(1)),
            ..row(5)
        };
        let resolved = resolve(vec![ambiguous, unknown, nameless, bad], &lookups());
        assert_eq!(resolved.cards, []);
        let reasons: Vec<(u64, UnmatchedReason)> = resolved
            .unmatched
            .iter()
            .map(|u| (u.line, u.reason))
            .collect();
        assert_eq!(
            reasons,
            [
                (2, UnmatchedReason::AmbiguousName),
                (3, UnmatchedReason::UnknownPrinting),
                (4, UnmatchedReason::MissingName),
                (5, UnmatchedReason::BadQuantity),
            ]
        );
        assert_eq!(resolved.unmatched_total, 4);
    }

    #[test]
    fn rows_on_one_key_merge_and_others_stay_apart() {
        let tagged = |tags: &str, copies: i32| ExportRow {
            scryfall_id: Some(Uuid::from_u128(1)),
            quantity: CollectionQuantity::new(copies).ok(),
            tags: CollectionTags::from_cell(tags).unwrap(),
            ..row(2)
        };
        let foil = ExportRow {
            finish: Finish::Foil,
            ..tagged("", 1)
        };
        let resolved = resolve(
            vec![tagged("trade", 2), tagged("Trade,deck", 9_998), foil],
            &lookups(),
        );
        let [nonfoil, foil] = resolved.cards.as_slice() else {
            panic!("{resolved:?}")
        };
        assert_eq!(
            nonfoil.quantity.get(),
            CollectionQuantity::MAX,
            "held at the cap"
        );
        let tags: Vec<&str> = nonfoil
            .tags
            .as_slice()
            .iter()
            .map(CollectionTag::as_str)
            .collect();
        assert_eq!(tags, ["trade", "deck"]);
        assert_eq!((foil.finish, foil.quantity.get()), (Finish::Foil, 1));
    }

    #[test]
    fn the_reply_lists_the_first_unmatched_and_counts_the_rest() {
        let rows: Vec<ExportRow> = (0..300)
            .map(|n| ExportRow {
                name: Some("Nowhere".into()),
                ..row(n + 2)
            })
            .collect();
        let resolved = resolve(rows, &lookups());
        assert_eq!(resolved.unmatched.len(), MAX_UNMATCHED_LISTED);
        assert_eq!(resolved.unmatched_total, 300);
    }

    #[test]
    fn wanted_asks_once_for_each_thing_and_lowercases_names() {
        let a = ExportRow {
            name: Some("Sol Ring".into()),
            set_code: Some("c21".into()),
            collector_number: Some("263".into()),
            scryfall_id: Some(Uuid::from_u128(1)),
            ..row(2)
        };
        let b = ExportRow {
            line: 3,
            ..a.clone()
        };
        let skipped = ExportRow {
            quantity: None,
            name: Some("Skip".into()),
            ..row(4)
        };
        let wanted = Wanted::of(&[a, b, skipped]);
        assert_eq!(wanted.ids, [Uuid::from_u128(1)]);
        assert_eq!(wanted.set_numbers, [("c21".to_owned(), "263".to_owned())]);
        assert_eq!(
            wanted.name_sets,
            [("sol ring".to_owned(), "c21".to_owned())]
        );
        assert_eq!(wanted.names, ["sol ring".to_owned()]);
    }
}
