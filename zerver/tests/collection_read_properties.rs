//! Property tests for reading a ManaBox or Archidekt collection export: generated files are
//! written with the csv crate under each app's real header and read back, at the parser alone,
//! with no database.

#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a test: generators unwrap and index, and a panic is the failure report"
)]

use chrono::{DateTime, Utc};
use proptest::prelude::*;
use uuid::Uuid;
use zwipe::domain::collection::models::export::{ExportRow, read};
use zwipe_core::domain::collection::limits::MAX_CELL_CHARS;
use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionQuantity, CollectionSource, CollectionTags, Finish,
    ImportCollectionError, InvalidCollectionTags, PurchaseCurrency, PurchasePrice,
};

/// ManaBox's export header, as it writes it.
const MANABOX: [&str; 18] = [
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
];

/// Archidekt's export header, as it writes it.
const ARCHIDEKT: [&str; 13] = [
    "Quantity",
    "Name",
    "Finish",
    "Condition",
    "Date Added",
    "Language",
    "Purchase Price",
    "Tags",
    "Edition Name",
    "Edition Code",
    "Multiverse Id",
    "Scryfall ID",
    "Collector Number",
];

/// Columns either app could write that name neither app: no ManaBox ID or Set code, no
/// Edition Code or Date Added.
const NEITHER: [&str; 8] = [
    "Name",
    "Quantity",
    "Scryfall ID",
    "Collector number",
    "Foil",
    "Condition",
    "Language",
    "Set name",
];

/// One card as an export holds it.
#[derive(Debug, Clone)]
struct Card {
    name: String,
    set_code: String,
    collector_number: String,
    quantity: i32,
    finish: Finish,
    condition: CardCondition,
    language: CardLanguage,
    scryfall_id: Option<Uuid>,
    price_cents: Option<i64>,
    tags: Vec<String>,
    added: i64,
}

fn card() -> impl Strategy<Value = Card> {
    (
        (
            "[A-Za-z][A-Za-z ,.'!\"-]{0,28}[A-Za-z]",
            "[a-z0-9]{3,5}",
            "[0-9]{1,4}[a-z]?",
            1..=CollectionQuantity::MAX,
        ),
        (
            prop::sample::select(vec![Finish::Nonfoil, Finish::Foil, Finish::Etched]),
            prop::sample::select(CardCondition::ALL.to_vec()),
            prop::sample::select(CardLanguage::ALL.to_vec()),
            prop::option::of(any::<u128>().prop_map(Uuid::from_u128)),
            prop::option::of(0i64..=99_999_999),
        ),
        (
            prop::collection::btree_set("[a-z][a-z ]{0,18}[a-z]", 0..5),
            0i64..2_000_000_000,
        ),
    )
        .prop_map(
            |(
                (name, set_code, collector_number, quantity),
                (finish, condition, language, scryfall_id, price_cents),
                (tags, added),
            )| Card {
                name,
                set_code,
                collector_number,
                quantity,
                finish,
                condition,
                language,
                scryfall_id,
                price_cents,
                tags: tags.into_iter().collect(),
                added,
            },
        )
}

/// A price as both apps write it.
fn price(cents: i64) -> String {
    format!(
        "{}.{:02}",
        cents.checked_div(100).unwrap(),
        cents.checked_rem(100).unwrap()
    )
}

/// The moment `seconds` past the epoch.
fn moment(seconds: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(seconds, 0).unwrap()
}

/// Archidekt's spelling of a condition.
fn archidekt_condition(condition: CardCondition) -> &'static str {
    match condition {
        CardCondition::Mint => "M",
        CardCondition::NearMint => "NM",
        CardCondition::Excellent => "EX",
        CardCondition::Good => "GD",
        CardCondition::LightlyPlayed => "LP",
        CardCondition::ModeratelyPlayed => "MP",
        CardCondition::Played => "PL",
        CardCondition::HeavilyPlayed => "HP",
        CardCondition::Poor => "PO",
        CardCondition::Damaged | CardCondition::Unknown => "D",
    }
}

/// `card`'s cell under `column` in `source`'s export.
fn cell(source: CollectionSource, column: &str, card: &Card) -> String {
    let finish = match card.finish {
        Finish::Foil => "foil",
        Finish::Etched => "etched",
        Finish::Nonfoil | Finish::Unknown => "normal",
    };
    let manabox = source == CollectionSource::ManaBox;
    match column {
        "Name" => card.name.clone(),
        "Set code" => card.set_code.to_uppercase(),
        "Edition Code" => card.set_code.clone(),
        "Collector number" | "Collector Number" => card.collector_number.clone(),
        "Foil" => finish.to_owned(),
        "Finish" => {
            let mut word = finish.to_owned();
            word[..1].make_ascii_uppercase();
            word
        }
        "Quantity" => card.quantity.to_string(),
        "Scryfall ID" => card
            .scryfall_id
            .map(|id| id.to_string())
            .unwrap_or_default(),
        "Purchase price" | "Purchase Price" => card.price_cents.map(price).unwrap_or_default(),
        "Purchase price currency" => "USD".to_owned(),
        "Condition" if manabox => card.condition.as_str().to_owned(),
        "Condition" => archidekt_condition(card.condition).to_owned(),
        "Language" if manabox => card.language.as_str().to_owned(),
        "Language" => card.language.as_str().to_uppercase(),
        "Tags" => card.tags.join(", "),
        "Added" => moment(card.added)
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string(),
        "Date Added" => moment(card.added).format("%Y-%m-%d").to_string(),
        "Misprint" | "Altered" | "Signed" | "Proxy" => "false".to_owned(),
        "ManaBox ID" | "Multiverse Id" => "12345".to_owned(),
        "Rarity" => "rare".to_owned(),
        _ => "Some Set".to_owned(),
    }
}

/// The row reading `card` from `source`'s export on `line` should give.
fn expected(source: CollectionSource, card: &Card, line: u64) -> ExportRow {
    let manabox = source == CollectionSource::ManaBox;
    let added = if manabox {
        moment(card.added)
    } else {
        moment(
            card.added
                .checked_div(86_400)
                .unwrap()
                .checked_mul(86_400)
                .unwrap(),
        )
    };
    let purchase_price = card
        .price_cents
        .map(|cents| PurchasePrice::from_cents(cents).unwrap());
    ExportRow {
        line,
        name: Some(card.name.clone()),
        scryfall_id: card.scryfall_id,
        set_code: Some(card.set_code.clone()),
        collector_number: Some(card.collector_number.clone()),
        quantity: Some(CollectionQuantity::new(card.quantity).unwrap()),
        finish: card.finish,
        condition: Some(card.condition),
        language: Some(card.language),
        tags: if manabox {
            CollectionTags::default()
        } else {
            CollectionTags::from_cell(&card.tags.join(", ")).unwrap()
        },
        purchase_price,
        purchase_currency: (manabox && purchase_price.is_some())
            .then(|| "USD".parse::<PurchaseCurrency>().unwrap()),
        added_at: Some(added),
    }
}

/// One CSV line holding `cells`, quoted where the csv crate quotes.
fn line(cells: &[String]) -> String {
    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    writer.write_record(cells).unwrap();
    String::from_utf8(writer.into_inner().unwrap())
        .unwrap()
        .trim_end_matches(['\r', '\n'])
        .to_owned()
}

/// `source`'s header, in the column order `order` (a permutation of its indices).
fn header(source: CollectionSource) -> Vec<&'static str> {
    if source == CollectionSource::ManaBox {
        MANABOX.to_vec()
    } else {
        ARCHIDEKT.to_vec()
    }
}

/// The lines of `source`'s export of `cards` with its columns in `columns` order.
fn export_lines(source: CollectionSource, columns: &[&str], cards: &[Card]) -> Vec<String> {
    let mut lines = vec![line(
        &columns.iter().map(|c| (*c).to_owned()).collect::<Vec<_>>(),
    )];
    for card in cards {
        lines.push(line(
            &columns
                .iter()
                .map(|column| cell(source, column, card))
                .collect::<Vec<_>>(),
        ));
    }
    lines
}

/// Either app, with its header's columns in any order.
fn app_and_columns() -> impl Strategy<Value = (CollectionSource, Vec<&'static str>)> {
    prop::sample::select(vec![CollectionSource::ManaBox, CollectionSource::Archidekt])
        .prop_flat_map(|source| (Just(source), Just(header(source)).prop_shuffle()))
}

proptest! {
    /// Guards the round trip: any rows either app writes, under its header in any column
    /// order, read back as that app with the same fields, in order, on the right lines.
    #[test]
    fn any_export_reads_back_field_for_field(
        (source, columns) in app_and_columns(),
        cards in prop::collection::vec(card(), 1..12),
    ) {
        let text = export_lines(source, &columns, &cards).join("\n");
        let export = read(&text).unwrap();
        prop_assert_eq!(export.source, source);
        let want: Vec<ExportRow> = cards
            .iter()
            .zip(2u64..)
            .map(|(card, line)| expected(source, card, line))
            .collect();
        prop_assert_eq!(export.rows, want);
    }

    /// Guards header detection: either app's header in any column order, letter case and
    /// padding, with or without a byte order mark, reads as that app and never the other; a
    /// header with neither app's marker columns is turned down.
    #[test]
    fn a_header_reads_as_its_own_app_and_never_the_other(
        (source, columns) in app_and_columns(),
        upper in prop::collection::vec(any::<bool>(), 18),
        pad in prop::collection::vec(0usize..3, 18),
        bom in any::<bool>(),
        card in card(),
        neither in prop::sample::subsequence(NEITHER.to_vec(), 2..=NEITHER.len()),
    ) {
        let written: Vec<String> = columns
            .iter()
            .zip(upper.iter().zip(&pad))
            .map(|(column, (up, pad))| {
                let cased = if *up { column.to_uppercase() } else { column.to_lowercase() };
                format!("{}{cased}{}", " ".repeat(*pad), " ".repeat(*pad))
            })
            .collect();
        let head = format!("{}{}", if bom { "\u{feff}" } else { "" }, line(&written));
        let row = line(&columns.iter().map(|c| cell(source, c, &card)).collect::<Vec<_>>());
        prop_assert_eq!(read(&format!("{head}\n{row}")).unwrap().source, source);

        let plain = line(&neither.iter().map(|c| (*c).to_owned()).collect::<Vec<_>>());
        prop_assert_eq!(
            read(&format!("{plain}\n{row}")),
            Err(ImportCollectionError::UnrecognizedHeader)
        );
    }

    /// Guards the quantity cell: any text gives the row its copies exactly when it is a whole
    /// number from 1 to 9,999, and otherwise no copies; it never fails the file.
    #[test]
    fn a_quantity_cell_lands_in_range_or_gives_no_copies(
        (source, columns) in app_and_columns(),
        card in card(),
        quantity in prop_oneof![
            any::<i64>().prop_map(|n| n.to_string()),
            "[ ]{0,2}[+-]?[0-9]{1,6}[ ]{0,2}",
            "[a-z0-9.+ ]{0,8}",
        ],
    ) {
        let cells: Vec<String> = columns
            .iter()
            .map(|column| if *column == "Quantity" { quantity.clone() } else { cell(source, column, &card) })
            .collect();
        let text = format!("{}\n{}", line(&columns.iter().map(|c| (*c).to_owned()).collect::<Vec<_>>()), line(&cells));
        let export = read(&text).unwrap();
        let want = quantity
            .trim()
            .parse::<i32>()
            .ok()
            .and_then(|n| CollectionQuantity::new(n).ok());
        prop_assert_eq!(export.rows[0].quantity, want);
    }

    /// Guards the reader against anything: any text, and any junk after a real header, reads
    /// to an export or one of the documented errors, never a panic, and every row it keeps sits
    /// on a line the text has.
    #[test]
    fn junk_never_panics(
        bytes in prop::collection::vec(any::<u8>(), 0..400),
        text in any::<String>(),
        (_source, columns) in app_and_columns(),
    ) {
        let lossy = String::from_utf8_lossy(&bytes).into_owned();
        let head = line(&columns.iter().map(|c| (*c).to_owned()).collect::<Vec<_>>());
        for input in [lossy.clone(), text.clone(), format!("{head}\n{lossy}"), format!("{head}\n{text}")] {
            if let Ok(export) = read(&input) {
                let lines = u64::try_from(input.lines().count()).unwrap();
                prop_assert!(export.rows.iter().all(|row| (2..=lines.max(2)).contains(&row.line)), "{export:?}");
            }
        }
    }

    /// Guards all or nothing: blank lines are skipped and every other line is a row, and one
    /// bad line anywhere (a cell past its cap, tags past theirs) turns the whole file down with
    /// that line named, never a part of it read.
    #[test]
    fn one_bad_line_turns_the_whole_file_down(
        (source, columns) in app_and_columns(),
        cards in prop::collection::vec(card(), 1..10),
        blanks in prop::collection::vec(any::<prop::sample::Index>(), 0..4),
        at in any::<prop::sample::Index>(),
        bad in 0usize..2,
    ) {
        let mut lines = export_lines(source, &columns, &cards);
        for blank in &blanks {
            lines.insert(blank.index(lines.len()).max(1), String::new());
        }
        let text = lines.join("\n");
        let export = read(&text).unwrap();
        let names: Vec<Option<String>> = export.rows.iter().map(|r| r.name.clone()).collect();
        let want: Vec<Option<String>> = cards.iter().map(|c| Some(c.name.clone())).collect();
        prop_assert_eq!(names, want, "every non-blank line is a row");

        let mut poisoned = cards[0].clone();
        let use_tags = bad == 1 && source == CollectionSource::Archidekt;
        if use_tags {
            poisoned.tags = (0..=CollectionTags::MAX).map(|i| format!("tag {i}")).collect();
        } else {
            poisoned.name = "x".repeat(MAX_CELL_CHARS.checked_add(1).unwrap());
        }
        let bad_line = line(&columns.iter().map(|c| cell(source, c, &poisoned)).collect::<Vec<_>>());
        let where_at = at.index(lines.len()).max(1);
        lines.insert(where_at, bad_line);
        let line_no = u64::try_from(where_at).unwrap().checked_add(1).unwrap();
        let want = if use_tags {
            ImportCollectionError::InvalidTags { line: line_no, reason: InvalidCollectionTags::TooMany }
        } else {
            ImportCollectionError::CellTooLong { line: line_no }
        };
        prop_assert_eq!(read(&lines.join("\n")), Err(want));
    }
}

/// Guards all or nothing for a cell that opens a quote and never closes it: the reader must not
/// fold the lines after it into that cell and keep the rest of the file.
#[test]
#[ignore = "waits for scotland's unclosed-quote fix"]
fn an_unclosed_quote_never_swallows_the_lines_after_it() {
    proptest!(|(
        (source, columns) in app_and_columns(),
        cards in prop::collection::vec(card(), 2..10),
        at in any::<prop::sample::Index>(),
    )| {
        let mut lines = export_lines(source, &columns, &cards);
        let row = at.index(cards.len()).checked_add(1).unwrap();
        let name_at = columns.iter().position(|c| *c == "Name").unwrap();
        let cells: Vec<String> = columns
            .iter()
            .enumerate()
            .map(|(i, c)| if i == name_at { format!("\"{}", cards[0].name.replace('"', "")) } else { cell(source, c, &cards[0]) })
            .collect();
        lines[row] = cells.join(",");
        match read(&lines.join("\n")) {
            Err(_) => {}
            Ok(export) => prop_assert_eq!(export.rows.len(), cards.len(), "a row per line, or an error"),
        }
    });
}
