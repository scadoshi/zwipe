//! Writing a collection out as a ManaBox export: ManaBox's own header and columns, then a Tags
//! column, which the collection import reads back from either app. A cell a spreadsheet would
//! run as a formula is written as text.

use chrono::{DateTime, SecondsFormat, Utc};
use csv::WriterBuilder;
use uuid::Uuid;
use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionQuantity, CollectionTag, CollectionTags, Finish,
    PurchaseCurrency, PurchasePrice,
};

/// The export's header: ManaBox's columns in ManaBox's order, then Tags.
pub const HEADER: [&str; 19] = [
    "Name",
    "Set code",
    "Set name",
    "Collector number",
    "Foil",
    "Rarity",
    "Quantity",
    "ManaBox ID",
    "Scryfall ID",
    "Purchase price",
    "Misprint",
    "Altered",
    "Signed",
    "Condition",
    "Language",
    "Proxy",
    "Purchase price currency",
    "Added",
    "Tags",
];

/// One collection row with the card facts its line carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportLine {
    /// The card's name.
    pub name: String,
    /// The printing's set code.
    pub set_code: String,
    /// The printing's set name.
    pub set_name: String,
    /// The printing's collector number.
    pub collector_number: String,
    /// The row's finish.
    pub finish: Finish,
    /// The printing's rarity, as ManaBox spells it (`rare`).
    pub rarity: String,
    /// The row's copies.
    pub quantity: CollectionQuantity,
    /// The printing's Scryfall id.
    pub scryfall_id: Uuid,
    /// What the row says was paid.
    pub purchase_price: Option<PurchasePrice>,
    /// The price's currency.
    pub purchase_currency: Option<PurchaseCurrency>,
    /// The row's condition.
    pub condition: Option<CardCondition>,
    /// The row's language.
    pub language: Option<CardLanguage>,
    /// When the row was added.
    pub added_at: Option<DateTime<Utc>>,
    /// The row's tags.
    pub tags: CollectionTags,
}

/// The characters that make a spreadsheet read a cell as a formula when it starts with one.
const FORMULA_STARTS: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];

/// `cell` as a spreadsheet shows it as text: with a `'` in front when it starts like a formula.
fn defused(cell: &str) -> String {
    if cell.starts_with(FORMULA_STARTS) {
        format!("'{cell}")
    } else {
        cell.to_owned()
    }
}

/// ManaBox's spelling of `finish`.
fn finish_cell(finish: Finish) -> &'static str {
    match finish {
        Finish::Foil => "foil",
        Finish::Etched => "etched",
        Finish::Nonfoil | Finish::Unknown => "normal",
    }
}

/// `price` in whole units with two decimals, as both apps write it (`0.41`).
fn price_cell(price: PurchasePrice) -> String {
    let cents = price.cents();
    let whole = cents.checked_div(100).unwrap_or_default();
    let rest = cents.checked_rem(100).unwrap_or_default();
    format!("{whole}.{rest:02}")
}

/// `tags` as one cell, the way Archidekt writes them (`trade, binder 2`).
fn tags_cell(tags: &CollectionTags) -> String {
    let names: Vec<&str> = tags.as_slice().iter().map(CollectionTag::as_str).collect();
    names.join(", ")
}

/// The export's text for `lines`, header first, one line per row in order.
///
/// # Errors
///
/// Returns the CSV writer's error, which writing to memory never gives in practice.
pub fn write(lines: &[ExportLine]) -> Result<String, anyhow::Error> {
    let mut writer = WriterBuilder::new().from_writer(Vec::new());
    writer.write_record(HEADER)?;
    for line in lines {
        let price = line.purchase_price.map_or_else(String::new, price_cell);
        let currency = line
            .purchase_currency
            .as_ref()
            .filter(|_| line.purchase_price.is_some())
            .map_or_else(String::new, |c| c.as_str().to_owned());
        let added = line.added_at.map_or_else(String::new, |at| {
            at.to_rfc3339_opts(SecondsFormat::Millis, true)
        });
        let record = [
            defused(&line.name),
            defused(&line.set_code.to_uppercase()),
            defused(&line.set_name),
            defused(&line.collector_number),
            finish_cell(line.finish).to_owned(),
            defused(&line.rarity),
            line.quantity.get().to_string(),
            String::new(),
            line.scryfall_id.to_string(),
            price,
            "false".to_owned(),
            "false".to_owned(),
            "false".to_owned(),
            line.condition.map_or("", CardCondition::as_str).to_owned(),
            line.language.map_or("", CardLanguage::as_str).to_owned(),
            "false".to_owned(),
            currency,
            added,
            defused(&tags_cell(&line.tags)),
        ];
        writer.write_record(&record)?;
    }
    let bytes = writer.into_inner().map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(String::from_utf8(bytes)?)
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::collection::CollectionSource;

    use super::*;
    use crate::domain::collection::models::export::read;

    fn line(name: &str) -> ExportLine {
        ExportLine {
            name: name.to_owned(),
            set_code: "cmr".to_owned(),
            set_name: "Commander Legends".to_owned(),
            collector_number: "266".to_owned(),
            finish: Finish::Nonfoil,
            rarity: "rare".to_owned(),
            quantity: CollectionQuantity::new(1).unwrap(),
            scryfall_id: Uuid::from_u128(1),
            purchase_price: None,
            purchase_currency: None,
            condition: None,
            language: None,
            added_at: None,
            tags: CollectionTags::default(),
        }
    }

    #[test]
    fn the_export_reads_back_field_for_field() {
        let full = ExportLine {
            name: "Amareth, the \"Lustrous\"".to_owned(),
            finish: Finish::Etched,
            quantity: CollectionQuantity::new(4).unwrap(),
            purchase_price: Some(PurchasePrice::from_cents(4105).unwrap()),
            purchase_currency: Some("EUR".parse().unwrap()),
            condition: Some(CardCondition::LightlyPlayed),
            language: Some(CardLanguage::ALL[6]),
            added_at: Some("2026-08-01T15:31:19.629Z".parse().unwrap()),
            tags: CollectionTags::from_cell("trade, binder 2").unwrap(),
            ..line("x")
        };
        let lines = [full, line("Sol Ring")];
        let export = read(&write(&lines).unwrap()).unwrap();
        assert_eq!(export.source, CollectionSource::ManaBox);
        assert_eq!(export.rows.len(), 2);
        for (row, want) in export.rows.iter().zip(&lines) {
            assert_eq!(row.name.as_deref(), Some(want.name.as_str()));
            assert_eq!(row.scryfall_id, Some(want.scryfall_id));
            assert_eq!(row.set_code.as_deref(), Some(want.set_code.as_str()));
            assert_eq!(
                row.collector_number.as_deref(),
                Some(want.collector_number.as_str())
            );
            assert_eq!(row.quantity, Some(want.quantity));
            assert_eq!(row.finish, want.finish);
            assert_eq!(row.condition, want.condition);
            assert_eq!(row.language, want.language);
            assert_eq!(row.purchase_price, want.purchase_price);
            assert_eq!(row.purchase_currency, want.purchase_currency);
            assert_eq!(row.added_at, want.added_at);
            assert_eq!(row.tags, want.tags);
        }
    }

    #[test]
    fn every_finish_condition_and_language_reads_back() {
        let mut lines = Vec::new();
        for (at, finish) in Finish::ALL.into_iter().enumerate() {
            for condition in CardCondition::ALL {
                for language in CardLanguage::ALL {
                    lines.push(ExportLine {
                        finish,
                        condition: Some(condition),
                        language: Some(language),
                        scryfall_id: Uuid::from_u128(u128::try_from(lines.len()).unwrap()),
                        collector_number: format!("{at}"),
                        ..line("Card")
                    });
                }
            }
        }
        let export = read(&write(&lines).unwrap()).unwrap();
        assert_eq!(export.rows.len(), lines.len());
        for (row, want) in export.rows.iter().zip(&lines) {
            assert_eq!(
                (row.finish, row.condition, row.language),
                (want.finish, want.condition, want.language)
            );
        }
    }

    #[test]
    fn a_cell_a_spreadsheet_would_run_is_written_as_text() {
        let hostile = ExportLine {
            name: "=HYPERLINK(\"http://x\",\"click\")".to_owned(),
            set_name: "+1+1".to_owned(),
            tags: CollectionTags::from_cell("@SUM(A1), -2+3").unwrap(),
            ..line("x")
        };
        let csv = write(&[hostile]).unwrap();
        let mut cells = csv::Reader::from_reader(csv.as_bytes());
        let record = cells.records().next().unwrap().unwrap();
        for cell in &record {
            assert!(
                !cell.starts_with(FORMULA_STARTS),
                "no cell starts like a formula: {cell:?} in {csv}"
            );
        }
        assert_eq!(record.get(0), Some("'=HYPERLINK(\"http://x\",\"click\")"));
        assert_eq!(record.get(2), Some("'+1+1"));
        assert_eq!(record.get(18), Some("'@SUM(A1), -2+3"));
        assert_eq!(defused("Sol Ring"), "Sol Ring", "plain text is untouched");
    }
}
