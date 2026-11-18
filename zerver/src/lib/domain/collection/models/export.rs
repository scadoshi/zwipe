//! Reading a ManaBox or Archidekt collection export: the header row says whose it is, and each
//! card row becomes an [`ExportRow`]. The reader goes a row at a time and keeps only the parsed
//! fields. Every cell loses its control characters; a cell past the cap, a row past the cap or
//! a tag list past its caps turns the whole file down. A value the reader doesn't know (a finish,
//! condition, language, price or date) is not given rather than an error, so one odd cell never
//! sinks a row.

use chrono::{DateTime, NaiveDate, Utc};
use csv::{ReaderBuilder, StringRecord};
use uuid::Uuid;
use zwipe_core::domain::collection::limits::{MAX_CELL_CHARS, MAX_ROWS};
use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionQuantity, CollectionSource, CollectionTags, Finish,
    ImportCollectionError, PurchaseCurrency, PurchasePrice,
};

/// A read export: whose it is and its card rows, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    /// Whose export it is.
    pub source: CollectionSource,
    /// The card rows, blank rows left out.
    pub rows: Vec<ExportRow>,
}

/// One card row, as read. Nothing here has been matched to a printing yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRow {
    /// Its line in the file, 1 for the header.
    pub line: u64,
    /// The card name.
    pub name: Option<String>,
    /// The printing's Scryfall id.
    pub scryfall_id: Option<Uuid>,
    /// The set code, lowercased as Scryfall writes it.
    pub set_code: Option<String>,
    /// The collector number.
    pub collector_number: Option<String>,
    /// The copies, or `None` for a quantity outside 1 to [`CollectionQuantity::MAX`].
    pub quantity: Option<CollectionQuantity>,
    /// The finish; an unread one is no foil.
    pub finish: Finish,
    /// The condition.
    pub condition: Option<CardCondition>,
    /// The language.
    pub language: Option<CardLanguage>,
    /// The tags.
    pub tags: CollectionTags,
    /// The purchase price.
    pub purchase_price: Option<PurchasePrice>,
    /// The purchase price's currency, only with a price.
    pub purchase_currency: Option<PurchaseCurrency>,
    /// When it was added to the collection in the app it came from.
    pub added_at: Option<DateTime<Utc>>,
}

/// Where each field sits in a row, by column index.
#[derive(Debug, Clone, Copy, Default)]
struct Columns {
    name: Option<usize>,
    scryfall_id: Option<usize>,
    set_code: Option<usize>,
    collector_number: Option<usize>,
    quantity: Option<usize>,
    finish: Option<usize>,
    condition: Option<usize>,
    language: Option<usize>,
    tags: Option<usize>,
    purchase_price: Option<usize>,
    purchase_currency: Option<usize>,
    added_at: Option<usize>,
}

impl Columns {
    /// The columns of a header row, and whose export it is. Headers match ignoring case,
    /// surrounding spaces and anything before the first letter (a byte order mark, or what a
    /// mis-read one turns into), each under every spelling either app uses.
    fn of(header: &StringRecord) -> Result<(CollectionSource, Self), ImportCollectionError> {
        let names: Vec<String> = header
            .iter()
            .map(|cell| {
                clean(cell)
                    .trim_start_matches(|c: char| !c.is_alphanumeric())
                    .trim()
                    .to_lowercase()
            })
            .collect();
        let find = |spellings: &[&str]| names.iter().position(|n| spellings.contains(&n.as_str()));
        let columns = Self {
            name: find(&["name", "card name"]),
            scryfall_id: find(&["scryfall id", "scryfall_id", "scryfallid"]),
            set_code: find(&["set code", "edition code", "set"]),
            collector_number: find(&["collector number", "card number", "number"]),
            quantity: find(&["quantity", "count", "qty"]),
            finish: find(&["foil", "finish"]),
            condition: find(&["condition"]),
            language: find(&["language", "lang"]),
            tags: find(&["tags"]),
            purchase_price: find(&["purchase price"]),
            purchase_currency: find(&["purchase price currency", "purchase currency"]),
            added_at: find(&["added", "date added"]),
        };
        let has = |spelling: &str| names.iter().any(|n| n == spelling);
        let source = if has("manabox id") || has("set code") {
            CollectionSource::ManaBox
        } else if has("edition code") || has("date added") {
            CollectionSource::Archidekt
        } else {
            return Err(ImportCollectionError::UnrecognizedHeader);
        };
        let identifies = columns.scryfall_id.is_some()
            || (columns.set_code.is_some() && columns.collector_number.is_some())
            || columns.name.is_some();
        if columns.quantity.is_none() || !identifies {
            return Err(ImportCollectionError::UnrecognizedHeader);
        }
        Ok((source, columns))
    }
}

/// `cell` without control characters.
fn clean(cell: &str) -> String {
    cell.chars().filter(|c| !c.is_control()).collect()
}

/// Reads `csv`, a whole export with its header row first.
///
/// # Errors
///
/// Returns [`ImportCollectionError`] for an empty file, a header neither app writes, no card
/// rows, more than [`MAX_ROWS`] card rows, a cell past [`MAX_CELL_CHARS`], a line the CSV reader
/// can't read, or a row's tags past their caps.
pub fn read(csv: &str) -> Result<Export, ImportCollectionError> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv.as_bytes());
    let mut records = reader.records();
    let header = match records.next() {
        None => return Err(ImportCollectionError::Empty),
        Some(Err(e)) => return Err(malformed(&e)),
        Some(Ok(header)) => header,
    };
    check_cells(&header, 1)?;
    if header.iter().all(|cell| cell.trim().is_empty()) {
        return Err(ImportCollectionError::Empty);
    }
    let (source, columns) = Columns::of(&header)?;
    let mut rows = Vec::new();
    for record in records {
        let record = record.map_err(|e| malformed(&e))?;
        let line = record.position().map_or(0, csv::Position::line);
        check_cells(&record, line)?;
        if record.iter().all(|cell| cell.trim().is_empty()) {
            continue;
        }
        if rows.len() >= MAX_ROWS {
            return Err(ImportCollectionError::TooManyRows);
        }
        rows.push(row(&record, columns, line)?);
    }
    if rows.is_empty() {
        return Err(ImportCollectionError::NoRows);
    }
    Ok(Export { source, rows })
}

/// The CSV reader's error as the line it stopped on.
fn malformed(error: &csv::Error) -> ImportCollectionError {
    ImportCollectionError::Malformed {
        line: error.position().map_or(0, csv::Position::line),
    }
}

/// Turns down a record holding a cell past [`MAX_CELL_CHARS`].
fn check_cells(record: &StringRecord, line: u64) -> Result<(), ImportCollectionError> {
    if record
        .iter()
        .any(|cell| cell.chars().count() > MAX_CELL_CHARS)
    {
        return Err(ImportCollectionError::CellTooLong { line });
    }
    Ok(())
}

/// One card row.
fn row(
    record: &StringRecord,
    columns: Columns,
    line: u64,
) -> Result<ExportRow, ImportCollectionError> {
    let cell = |at: Option<usize>| -> Option<String> {
        let text = clean(record.get(at?)?);
        let text = text.trim();
        (!text.is_empty()).then(|| text.to_owned())
    };
    let tags = match cell(columns.tags) {
        Some(text) => CollectionTags::from_cell(&text)
            .map_err(|reason| ImportCollectionError::InvalidTags { line, reason })?,
        None => CollectionTags::default(),
    };
    let purchase_price = cell(columns.purchase_price).and_then(|text| text.parse().ok());
    Ok(ExportRow {
        line,
        name: cell(columns.name),
        scryfall_id: cell(columns.scryfall_id).and_then(|text| Uuid::try_parse(&text).ok()),
        set_code: cell(columns.set_code).map(|text| text.to_lowercase()),
        collector_number: cell(columns.collector_number),
        quantity: cell(columns.quantity)
            .and_then(|text| text.parse::<i32>().ok())
            .and_then(|copies| CollectionQuantity::new(copies).ok()),
        finish: cell(columns.finish)
            .and_then(|text| text.parse().ok())
            .unwrap_or_default(),
        condition: cell(columns.condition).and_then(|text| text.parse().ok()),
        language: cell(columns.language).and_then(|text| text.parse().ok()),
        tags,
        purchase_currency: purchase_price
            .and_then(|_| cell(columns.purchase_currency))
            .and_then(|text| text.parse().ok()),
        purchase_price,
        added_at: cell(columns.added_at).and_then(|text| added_at(&text)),
    })
}

/// A date added: ManaBox writes a full timestamp (`2026-08-01T15:31:19.629Z`), Archidekt a day
/// (`2025-03-03`), which reads as that day's midnight UTC.
fn added_at(text: &str) -> Option<DateTime<Utc>> {
    if let Ok(at) = DateTime::parse_from_rfc3339(text) {
        return Some(at.with_timezone(&Utc));
    }
    NaiveDate::parse_from_str(text, "%Y-%m-%d")
        .ok()
        .and_then(|day| day.and_hms_opt(0, 0, 0))
        .map(|at| at.and_utc())
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::collection::CollectionTag;

    use super::*;

    const MANABOX: &str = "Name,Set code,Set name,Collector number,Foil,Rarity,Quantity,ManaBox ID,Scryfall ID,Purchase price,Misprint,Altered,Signed,Condition,Language,Proxy,Purchase price currency,Added
\"Amareth, the Lustrous\",CMR,Commander Legends,266,normal,rare,1,55617,2fbea010-de77-468e-be39-1717070c9303,0.41,false,false,false,near_mint,en,false,USD,2026-08-01T15:31:19.629Z
Meren of Clan Nel Toth,TDC,Tarkir: Dragonstorm Commander,297,foil,mythic,2,104639,508b1442-bf2c-4ad6-9bcf-bd894e081ab6,,false,false,false,near_mint,en,false,USD,2026-08-01T15:31:19.630Z
";

    const ARCHIDEKT: &str = "Quantity,Name,Finish,Condition,Date Added,Language,Purchase Price,Tags,Edition Name,Edition Code,Multiverse Id,Scryfall ID,Collector Number
1,Aardwolf's Advantage,Normal,NM,2025-03-03,EN,,\"trade, binder 2\",Unfinity,unf,580777,c775a7d1-a1f7-460b-b1f4-888006058e15,97
3,Abattoir Ghoul,Etched,LP,2025-03-03,JP,1.50,,Duel Decks: Blessed vs. Cursed,ddq,409626,43297ddd-276f-4d2d-9d9c-937603d0e376,50
";

    #[test]
    fn a_manabox_export_reads_every_field() {
        let export = read(MANABOX).unwrap();
        assert_eq!(export.source, CollectionSource::ManaBox);
        let [amareth, meren] = export.rows.as_slice() else {
            panic!("two rows: {export:?}")
        };
        assert_eq!(amareth.line, 2);
        assert_eq!(amareth.name.as_deref(), Some("Amareth, the Lustrous"));
        assert_eq!(
            amareth.scryfall_id,
            Some("2fbea010-de77-468e-be39-1717070c9303".parse().unwrap())
        );
        assert_eq!(amareth.set_code.as_deref(), Some("cmr"));
        assert_eq!(amareth.collector_number.as_deref(), Some("266"));
        assert_eq!(amareth.quantity.map(CollectionQuantity::get), Some(1));
        assert_eq!(amareth.finish, Finish::Nonfoil);
        assert_eq!(amareth.condition, Some(CardCondition::NearMint));
        assert_eq!(amareth.language, Some(CardLanguage::En));
        assert_eq!(amareth.purchase_price.map(PurchasePrice::cents), Some(41));
        assert_eq!(
            amareth
                .purchase_currency
                .as_ref()
                .map(PurchaseCurrency::as_str),
            Some("USD")
        );
        assert_eq!(
            amareth.added_at.map(|at| at.to_rfc3339()),
            Some("2026-08-01T15:31:19.629+00:00".to_owned())
        );
        assert!(amareth.tags.is_empty());
        assert_eq!(meren.finish, Finish::Foil);
        assert_eq!(meren.quantity.map(CollectionQuantity::get), Some(2));
        assert_eq!(meren.purchase_price, None);
        assert_eq!(meren.purchase_currency, None, "no currency without a price");
    }

    #[test]
    fn an_archidekt_export_reads_every_field() {
        let export = read(ARCHIDEKT).unwrap();
        assert_eq!(export.source, CollectionSource::Archidekt);
        let [aardwolf, ghoul] = export.rows.as_slice() else {
            panic!("two rows: {export:?}")
        };
        assert_eq!(aardwolf.set_code.as_deref(), Some("unf"));
        assert_eq!(aardwolf.condition, Some(CardCondition::NearMint));
        assert_eq!(aardwolf.language, Some(CardLanguage::En));
        let tags: Vec<&str> = aardwolf
            .tags
            .as_slice()
            .iter()
            .map(CollectionTag::as_str)
            .collect();
        assert_eq!(tags, ["trade", "binder 2"]);
        assert_eq!(
            aardwolf.added_at.map(|at| at.to_rfc3339()),
            Some("2025-03-03T00:00:00+00:00".to_owned())
        );
        assert_eq!(ghoul.finish, Finish::Etched);
        assert_eq!(ghoul.condition, Some(CardCondition::LightlyPlayed));
        assert_eq!(ghoul.language, Some(CardLanguage::Ja));
        assert_eq!(ghoul.quantity.map(CollectionQuantity::get), Some(3));
        assert_eq!(ghoul.purchase_price.map(PurchasePrice::cents), Some(150));
    }

    #[test]
    fn odd_cells_are_not_given_and_never_sink_the_row() {
        let csv = "Quantity,Name,Finish,Condition,Date Added,Language,Purchase Price,Tags,Edition Code,Scryfall ID,Collector Number
0,Zero Card,Gilded,Shiny,yesterday,Klingon,=1+1,,abc,not-a-uuid,1
";
        let export = read(csv).unwrap();
        let [row] = export.rows.as_slice() else {
            panic!("{export:?}")
        };
        assert_eq!(row.quantity, None, "zero copies is not a quantity");
        assert_eq!(
            (row.finish, row.condition, row.language),
            (Finish::Nonfoil, None, None)
        );
        assert_eq!(
            (row.purchase_price, row.added_at, row.scryfall_id),
            (None, None, None)
        );
    }

    #[test]
    fn control_characters_never_reach_a_field() {
        let csv = "Name,Set code,Collector number,Quantity,Scryfall ID\n\"Sol\u{0} Ring\u{1b}\",c\u{7}21,1,1,\n";
        let export = read(csv).unwrap();
        let [row] = export.rows.as_slice() else {
            panic!("{export:?}")
        };
        assert_eq!(row.name.as_deref(), Some("Sol Ring"));
        assert_eq!(row.set_code.as_deref(), Some("c21"));
    }

    #[test]
    fn a_file_it_cant_take_is_turned_down_whole() {
        assert_eq!(read(""), Err(ImportCollectionError::Empty));
        assert_eq!(read(",,\n"), Err(ImportCollectionError::Empty));
        assert_eq!(
            read("hello,world\n1,2\n"),
            Err(ImportCollectionError::UnrecognizedHeader)
        );
        assert_eq!(
            read("Name,Set code\nSol Ring,c21\n"),
            Err(ImportCollectionError::UnrecognizedHeader),
            "no quantity"
        );
        assert_eq!(
            read("Name,Set code,Quantity\n"),
            Err(ImportCollectionError::NoRows)
        );
        assert_eq!(
            read("Name,Set code,Quantity\n,,\n"),
            Err(ImportCollectionError::NoRows)
        );
        let long = format!(
            "Name,Set code,Quantity\n{},c21,1\n",
            "x".repeat(MAX_CELL_CHARS + 1)
        );
        assert_eq!(
            read(&long),
            Err(ImportCollectionError::CellTooLong { line: 2 })
        );
        let tags = format!(
            "Quantity,Name,Tags,Edition Code,Collector Number\n1,Sol Ring,\"{}\",c21,1\n",
            (0..=CollectionTags::MAX)
                .map(|n| format!("t{n}"))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert!(matches!(
            read(&tags),
            Err(ImportCollectionError::InvalidTags { line: 2, .. })
        ));
    }

    #[test]
    fn more_rows_than_the_cap_are_turned_down() {
        let mut csv = String::from("Name,Set code,Quantity\n");
        for _ in 0..=MAX_ROWS {
            csv.push_str("a,b,1\n");
        }
        assert_eq!(read(&csv), Err(ImportCollectionError::TooManyRows));
        let at_cap = csv
            .split_once("a,b,1\n")
            .map(|(head, rest)| format!("{head}{rest}"))
            .unwrap();
        assert_eq!(read(&at_cap).map(|e| e.rows.len()), Ok(MAX_ROWS));
    }

    #[test]
    fn a_byte_order_mark_and_odd_header_case_still_read() {
        let csv = "\u{feff}NAME , set CODE,QUANTITY\nSol Ring,C21,1\n";
        assert_eq!(read(csv).map(|e| e.source), Ok(CollectionSource::ManaBox));
    }
}
