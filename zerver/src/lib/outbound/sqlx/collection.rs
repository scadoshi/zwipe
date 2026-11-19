//! Collection repository: matching export rows to printings, writing an import in one
//! transaction, giving one row its copies, and reading the rows a page at a time. Every value
//! is bound; no export or search text reaches SQL text.

use std::collections::HashMap;

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::rarity::Rarity;
use zwipe_core::domain::collection::limits::MAX_ROWS;
use zwipe_core::domain::collection::{
    CardCondition, CardLanguage, CollectionPageSize, CollectionQuantity, CollectionSearch,
    CollectionTag, CollectionTags, Finish, PurchaseCurrency, PurchasePrice,
};
use zwipe_core::domain::deck::ImportMode;
use zwipe_core::http::contracts::collection::HttpCollectionSummary;

use crate::domain::collection::models::export_file::ExportLine;
use crate::domain::collection::models::import::{
    Applied, CollectionCard, ImportOutcome, ImportRequest, Lookups, NameMatch, Printing, Wanted,
};
use crate::domain::collection::models::rows::{
    ExportFailure, SetRow, SetRowFailure, SortKey, StoredPage, StoredRow,
};
use crate::domain::collection::ports::CollectionRepository;
use crate::outbound::sqlx::postgres::Postgres;

/// Joins a row's tags for the bulk write. Tags never hold control characters, so the unit
/// separator can't appear inside one.
const TAG_SEPARATOR: char = '\u{1f}';

/// A printing found by a key: the key's two parts, its id and its oracle id.
type KeyedPrinting = (String, String, Uuid, Uuid);

/// The name-within-set and name-alone matches among `rows` (`(lowercased name, set, id, oracle
/// id)`): one printing is a match, more is [`NameMatch::Many`].
fn name_matches(
    rows: Vec<(String, String, Uuid, Uuid)>,
) -> (
    HashMap<(String, String), NameMatch>,
    HashMap<String, NameMatch>,
) {
    let mut by_name_set: HashMap<(String, String), NameMatch> = HashMap::new();
    let mut by_name: HashMap<String, NameMatch> = HashMap::new();
    for (name, set, id, oracle_id) in rows {
        let printing = Printing { id, oracle_id };
        by_name
            .entry(name.clone())
            .and_modify(|held| *held = NameMatch::Many)
            .or_insert(NameMatch::One(printing));
        by_name_set
            .entry((name, set))
            .and_modify(|held| *held = NameMatch::Many)
            .or_insert(NameMatch::One(printing));
    }
    (by_name_set, by_name)
}

impl CollectionRepository for Postgres {
    async fn fetch_printings(&self, wanted: &Wanted) -> Result<Lookups, anyhow::Error> {
        let mut lookups = Lookups::default();
        if !wanted.ids.is_empty() {
            let rows: Vec<(Uuid, Uuid)> = query_as(
                "SELECT id, oracle_id FROM scryfall_data \
                 WHERE id = ANY($1) AND oracle_id IS NOT NULL",
            )
            .bind(&wanted.ids)
            .fetch_all(&self.pool)
            .await
            .context("collection printings by id")?;
            lookups.by_id = rows
                .into_iter()
                .map(|(id, oracle_id)| (id, Printing { id, oracle_id }))
                .collect();
        }
        if !wanted.set_numbers.is_empty() {
            let (sets, numbers): (Vec<&str>, Vec<&str>) = wanted
                .set_numbers
                .iter()
                .map(|(set, number)| (set.as_str(), number.as_str()))
                .unzip();
            let rows: Vec<KeyedPrinting> = query_as(
                "SELECT DISTINCT ON (sd.set, sd.collector_number) \
                     sd.set, sd.collector_number, sd.id, sd.oracle_id \
                 FROM scryfall_data sd \
                 JOIN UNNEST($1::text[], $2::text[]) AS w(set, number) \
                   ON sd.set = w.set AND sd.collector_number = w.number \
                 WHERE sd.oracle_id IS NOT NULL \
                 ORDER BY sd.set, sd.collector_number, (sd.lang IS DISTINCT FROM 'en'), sd.id",
            )
            .bind(&sets)
            .bind(&numbers)
            .fetch_all(&self.pool)
            .await
            .context("collection printings by set and number")?;
            lookups.by_set_number = rows
                .into_iter()
                .map(|(set, number, id, oracle_id)| ((set, number), Printing { id, oracle_id }))
                .collect();
        }
        if !wanted.names.is_empty() {
            // Every printing of the wanted names in one indexed lookup; the name-within-set and
            // name-alone matches are both counted from it.
            let rows: Vec<(String, String, Uuid, Uuid)> = query_as(
                "SELECT lower(name), set, id, oracle_id FROM scryfall_data \
                 WHERE lower(name) = ANY($1) AND oracle_id IS NOT NULL",
            )
            .bind(&wanted.names)
            .fetch_all(&self.pool)
            .await
            .context("collection printings by name")?;
            let (by_name_set, by_name) = name_matches(rows);
            lookups.by_name_set = by_name_set;
            lookups.by_name = by_name;
        }
        Ok(lookups)
    }

    async fn apply_import(
        &self,
        user_id: Uuid,
        mode: ImportMode,
        cards: &[CollectionCard],
        request: Option<&ImportRequest>,
    ) -> Result<ImportOutcome, anyhow::Error> {
        let arrays = ImportArrays::of(cards);
        let mut tx = self.pool.begin().await.context("collection import begin")?;
        // One import at a time per collection, held to the end of the transaction. An advisory
        // lock keyed on the user, so the user's own row stays free for their other writes. It
        // also orders a repeat behind the import it repeats, so it finds that one's record.
        query("SELECT pg_advisory_xact_lock(hashtext('collection_import'), hashtext($1::text))")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .context("collection import lock")?;
        if let Some(request) = request
            && let Some(earlier) = earlier_import(&mut tx, user_id, request).await?
        {
            tx.rollback().await.context("collection import rollback")?;
            return Ok(earlier);
        }
        let removed = if mode.is_replace() {
            query(REMOVE_MISSING)
                .bind(user_id)
                .bind(&arrays.ids)
                .bind(&arrays.finishes)
                .bind(&arrays.conditions)
                .bind(&arrays.languages)
                .execute(&mut *tx)
                .await
                .context("collection import remove")?
                .rows_affected()
        } else {
            0
        };
        let inserted: Vec<bool> = query_scalar(WRITE)
            .bind(user_id)
            .bind(&arrays.ids)
            .bind(&arrays.oracle_ids)
            .bind(&arrays.finishes)
            .bind(&arrays.conditions)
            .bind(&arrays.languages)
            .bind(&arrays.quantities)
            .bind(&arrays.tags)
            .bind(&arrays.cents)
            .bind(&arrays.currencies)
            .bind(&arrays.added_ats)
            .bind(mode.is_replace())
            .fetch_all(&mut *tx)
            .await
            .context("collection import write")?;
        let cards_after: i64 = query_scalar(
            "SELECT COALESCE(sum(quantity), 0)::bigint FROM collection_cards WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .context("collection import count")?;
        let added = inserted.iter().filter(|fresh| **fresh).count();
        let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        let applied = Applied {
            added: count(added),
            updated: count(inserted.len().saturating_sub(added)),
            removed: u32::try_from(removed).unwrap_or(u32::MAX),
            cards: cards_after,
        };
        if let Some(request) = request {
            record_import(&mut tx, user_id, request, applied).await?;
        }
        tx.commit().await.context("collection import commit")?;
        Ok(ImportOutcome::Applied(applied))
    }

    async fn fetch_summary(&self, user_id: Uuid) -> Result<HttpCollectionSummary, anyhow::Error> {
        let (cards, printings): (i64, i64) = query_as(
            "SELECT count(DISTINCT oracle_id), count(DISTINCT scryfall_data_id) \
             FROM collection_cards WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .context("collection summary")?;
        Ok(HttpCollectionSummary { cards, printings })
    }

    async fn set_row(&self, row: &SetRow) -> Result<Option<StoredRow>, SetRowFailure> {
        let condition = row.condition.map(CardCondition::as_str);
        let language = row.language.map(CardLanguage::as_str);
        let mut tx = self.pool.begin().await.context("collection set begin")?;
        query(LOCK)
            .bind(row.user_id)
            .execute(&mut *tx)
            .await
            .context("collection set lock")?;
        let Some(copies) = row.quantity.row() else {
            query(REMOVE_ROW)
                .bind(row.user_id)
                .bind(row.scryfall_data_id)
                .bind(row.finish.as_str())
                .bind(condition)
                .bind(language)
                .execute(&mut *tx)
                .await
                .context("collection set remove")?;
            tx.commit().await.context("collection set commit")?;
            return Ok(None);
        };
        // The oracle id comes from the printing, so a printing with no card writes nothing. The
        // text is built from constants only; every value is bound.
        let written: Option<RowColumns> = query_as(AssertSqlSafe(format!(
            "INSERT INTO collection_cards AS cc \
                 (user_id, scryfall_data_id, oracle_id, finish, condition, language, quantity) \
             SELECT $1, sd.id, sd.oracle_id, $3, $4, $5, $6 FROM scryfall_data sd \
             WHERE sd.id = $2 AND sd.oracle_id IS NOT NULL \
             ON CONFLICT (user_id, scryfall_data_id, finish, condition, language) DO UPDATE SET \
                 quantity = EXCLUDED.quantity, updated_at = now() \
             RETURNING {ROW_COLUMNS}"
        )))
        .bind(row.user_id)
        .bind(row.scryfall_data_id)
        .bind(row.finish.as_str())
        .bind(condition)
        .bind(language)
        .bind(copies.get())
        .fetch_optional(&mut *tx)
        .await
        .context("collection set write")?;
        let Some(written) = written else {
            return Err(SetRowFailure::UnknownPrinting);
        };
        tx.commit().await.context("collection set commit")?;
        Ok(Some(stored_row(written)?))
    }

    async fn fetch_page(
        &self,
        user_id: Uuid,
        name: &CollectionSearch,
        after: Option<&SortKey>,
        limit: CollectionPageSize,
    ) -> Result<StoredPage, anyhow::Error> {
        // Built from constants only; the user and the search are bound.
        let total: i64 = query_scalar(AssertSqlSafe(format!("SELECT count(*) {PAGE_MATCH}")))
            .bind(user_id)
            .bind(name.as_str())
            .fetch_one(&self.pool)
            .await
            .context("collection page count")?;
        // One past the page, so a page that has rows after it is told apart. The name bound
        // first lets the list index start at the cursor; the whole key then picks the rows after
        // it.
        let wanted = i64::from(limit.get()).saturating_add(1);
        let after_clause = if after.is_some() {
            format!(
                " AND cc.card_name >= $4 AND ({SORT_KEY}) > ($4, $5, $6::numeric, $7, $8, $9, $10, $11)"
            )
        } else {
            String::new()
        };
        // Built from constants only; every value is bound.
        let sql = format!(
            "SELECT {ROW_COLUMNS}, {SORT_KEY_COLUMNS} {PAGE_MATCH}{after_clause} \
             ORDER BY {SORT_KEY} LIMIT $3"
        );
        let mut page = query_as::<_, PageColumns>(AssertSqlSafe(sql))
            .bind(user_id)
            .bind(name.as_str())
            .bind(wanted);
        if let Some(key) = after {
            page = page
                .bind(&key.card_name)
                .bind(&key.set_code)
                .bind(&key.number)
                .bind(&key.collector_number)
                .bind(&key.finish)
                .bind(&key.condition)
                .bind(&key.language)
                .bind(key.scryfall_data_id);
        }
        let mut columns: Vec<(RowColumns, SortKey)> = page
            .fetch_all(&self.pool)
            .await
            .context("collection page")?
            .into_iter()
            .map(page_row)
            .collect();
        let more = columns.len() > usize::try_from(limit.get()).unwrap_or(usize::MAX);
        columns.truncate(usize::try_from(limit.get()).unwrap_or(usize::MAX));
        let next = if more {
            columns.last().map(|(_, key)| key.clone())
        } else {
            None
        };
        let rows = columns
            .into_iter()
            .map(|(row, _)| stored_row(row))
            .collect::<Result<_, _>>()?;
        Ok(StoredPage { rows, total, next })
    }

    async fn fetch_export_lines(&self, user_id: Uuid) -> Result<Vec<ExportLine>, ExportFailure> {
        // One past the cap, so a collection too big to import back is told apart without
        // reading all of it.
        let cap = i64::try_from(MAX_ROWS)
            .unwrap_or(i64::MAX)
            .saturating_add(1);
        // Built from constants only; the user and the cap are bound.
        let rows: Vec<ExportColumns> = query_as(AssertSqlSafe(format!(
            "SELECT sd.name, sd.set, sd.set_name, sd.collector_number, sd.rarity, \
                 {ROW_COLUMNS}, (cc.purchase_price * 100)::bigint, cc.purchase_currency \
             FROM collection_cards cc JOIN scryfall_data sd ON sd.id = cc.scryfall_data_id \
             WHERE cc.user_id = $1 {ROW_ORDER} LIMIT $2"
        )))
        .bind(user_id)
        .bind(cap)
        .fetch_all(&self.pool)
        .await
        .context("collection export")?;
        if rows.len() > MAX_ROWS {
            return Err(ExportFailure::TooManyRows);
        }
        rows.into_iter()
            .map(export_line)
            .collect::<Result<_, _>>()
            .map_err(ExportFailure::Database)
    }

    async fn fetch_owned_printing_ids(
        &self,
        user_id: Uuid,
        oracle_id: Uuid,
    ) -> Result<Vec<Uuid>, anyhow::Error> {
        query_scalar(
            "SELECT DISTINCT scryfall_data_id FROM collection_cards \
             WHERE user_id = $1 AND oracle_id = $2",
        )
        .bind(user_id)
        .bind(oracle_id)
        .fetch_all(&self.pool)
        .await
        .context("owned printings")
    }
}

/// Takes the per-user collection lock to the end of the transaction, so an import and an edit of
/// one collection run one after the other.
const LOCK: &str =
    "SELECT pg_advisory_xact_lock(hashtext('collection_import'), hashtext($1::text))";

/// A stored row's columns, in [`ROW_COLUMNS`]' order.
type RowColumns = (
    Uuid,
    String,
    Option<String>,
    Option<String>,
    i32,
    Vec<String>,
    Option<DateTime<Utc>>,
);

/// The columns a [`StoredRow`] reads, prefixed `cc.`.
const ROW_COLUMNS: &str = "cc.scryfall_data_id, cc.finish, cc.condition, cc.language, cc.quantity, \
     cc.tags, cc.added_at";

/// A set call's removal: the one row with this key.
const REMOVE_ROW: &str = "DELETE FROM collection_cards WHERE user_id = $1 AND scryfall_data_id = $2 \
     AND finish = $3 AND condition IS NOT DISTINCT FROM $4 AND language IS NOT DISTINCT FROM $5";

/// The rows a list's search matches: the user's, whose stored folded card names hold `$2` folded
/// the same way (lower case, accents off), every row when `$2` is empty. `strpos`, not `LIKE`, so
/// `%` and `_` are text. Only the user's rows are read, through the list index.
const PAGE_MATCH: &str = "FROM collection_cards cc \
     WHERE cc.user_id = $1 \
       AND ($2 = '' OR strpos(cc.search_name, lower(f_unaccent($2))) > 0)";

/// A row's place in the list, the order [`SortKey`] describes, from the row's stored columns.
const SORT_KEY: &str = "cc.card_name, cc.set_code, \
     COALESCE(substring(cc.collector_number FROM '^[0-9]+')::numeric, 'Infinity'), \
     cc.collector_number, cc.finish, COALESCE(cc.condition, ''), COALESCE(cc.language, ''), \
     cc.scryfall_data_id";

/// [`SORT_KEY`] as columns a [`SortKey`] reads, the number as text and without the printing,
/// which [`ROW_COLUMNS`] already holds.
const SORT_KEY_COLUMNS: &str = "cc.card_name, cc.set_code, \
     COALESCE(substring(cc.collector_number FROM '^[0-9]+')::numeric, 'Infinity')::text, \
     cc.collector_number, cc.finish, COALESCE(cc.condition, ''), COALESCE(cc.language, '')";

/// A page row: [`ROW_COLUMNS`], then [`SORT_KEY_COLUMNS`] but the printing, which the row's own
/// first column already holds.
type PageColumns = (
    Uuid,
    String,
    Option<String>,
    Option<String>,
    i32,
    Vec<String>,
    Option<DateTime<Utc>>,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
);

/// A page row's stored columns and its place in the list.
fn page_row(columns: PageColumns) -> (RowColumns, SortKey) {
    let (
        id,
        finish,
        condition,
        language,
        quantity,
        tags,
        added_at,
        card_name,
        set_code,
        number,
        collector_number,
        finish_key,
        condition_key,
        language_key,
    ) = columns;
    let key = SortKey {
        card_name,
        set_code,
        number,
        collector_number,
        finish: finish_key,
        condition: condition_key,
        language: language_key,
        scryfall_data_id: id,
    };
    (
        (id, finish, condition, language, quantity, tags, added_at),
        key,
    )
}

/// The one variant of `all` whose stored spelling (`as_str`) is `text`.
fn stored<T: Copy>(all: &[T], as_str: fn(T) -> &'static str, text: &str) -> Option<T> {
    all.iter().copied().find(|value| as_str(*value) == text)
}

/// A stored row from its columns. The table's checks hold every value to a known one.
fn stored_row(columns: RowColumns) -> Result<StoredRow, anyhow::Error> {
    let (scryfall_data_id, finish, condition, language, quantity, tags, added_at) = columns;
    let finish = stored(&Finish::ALL, Finish::as_str, &finish)
        .with_context(|| format!("stored finish {finish:?}"))?;
    let condition = condition
        .map(|text| {
            stored(&CardCondition::ALL, CardCondition::as_str, &text)
                .with_context(|| format!("stored condition {text:?}"))
        })
        .transpose()?;
    let language = language
        .map(|text| {
            stored(&CardLanguage::ALL, CardLanguage::as_str, &text)
                .with_context(|| format!("stored language {text:?}"))
        })
        .transpose()?;
    let quantity = CollectionQuantity::new(quantity).context("stored quantity")?;
    let tags: Vec<CollectionTag> = tags
        .iter()
        .map(|tag| CollectionTag::new(tag).context("stored tag"))
        .collect::<Result<_, _>>()?;
    let tags = CollectionTags::try_from(tags).context("stored tags")?;
    Ok(StoredRow {
        scryfall_data_id,
        finish,
        condition,
        language,
        quantity,
        tags,
        added_at,
    })
}

/// The list's order, which the export keeps too.
const ROW_ORDER: &str = "ORDER BY cc.card_name, cc.set_code, \
     COALESCE(substring(cc.collector_number FROM '^[0-9]+')::numeric, 'Infinity'), \
     cc.collector_number, cc.finish, COALESCE(cc.condition, ''), COALESCE(cc.language, ''), \
     cc.scryfall_data_id";

/// An export line's columns, in the export query's order: the card's name, set code, set name,
/// collector number and rarity, then [`ROW_COLUMNS`], then the price in cents and its currency.
type ExportColumns = (
    String,
    String,
    String,
    String,
    Option<String>,
    Uuid,
    String,
    Option<String>,
    Option<String>,
    i32,
    Vec<String>,
    Option<DateTime<Utc>>,
    Option<i64>,
    Option<String>,
);

/// An export line from its columns.
fn export_line(columns: ExportColumns) -> Result<ExportLine, anyhow::Error> {
    let (
        name,
        set_code,
        set_name,
        collector_number,
        rarity,
        id,
        finish,
        condition,
        language,
        quantity,
        tags,
        added_at,
        cents,
        currency,
    ) = columns;
    let row = stored_row((id, finish, condition, language, quantity, tags, added_at))?;
    let rarity = rarity
        .as_deref()
        .and_then(|code| Rarity::try_from(code).ok())
        .map_or_else(String::new, |rarity| rarity.to_long_name().to_lowercase());
    let purchase_price = cents
        .map(PurchasePrice::from_cents)
        .transpose()
        .context("stored purchase price")?;
    let purchase_currency = currency
        .map(|code| code.parse::<PurchaseCurrency>())
        .transpose()
        .map_err(|_| anyhow::anyhow!("stored purchase currency"))?;
    Ok(ExportLine {
        name,
        set_code,
        set_name,
        collector_number,
        finish: row.finish,
        rarity,
        quantity: row.quantity,
        scryfall_id: row.scryfall_data_id,
        purchase_price,
        purchase_currency,
        condition: row.condition,
        language: row.language,
        added_at: row.added_at,
        tags: row.tags,
    })
}

/// What an import with `request`'s id in the last day did, after deleting `user_id`'s records
/// older than a day: the same import repeated, or the id reused for another. `None` when no
/// import used it.
async fn earlier_import(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    request: &ImportRequest,
) -> Result<Option<ImportOutcome>, anyhow::Error> {
    query(
        "DELETE FROM collection_import_requests \
         WHERE user_id = $1 AND created_at <= now() - interval '1 day'",
    )
    .bind(user_id)
    .execute(&mut **tx)
    .await
    .context("collection import requests expire")?;
    let earlier: Option<(Vec<u8>, i32, i32, i32, i64)> = query_as(
        "SELECT body_sha256, added, updated, removed, cards \
         FROM collection_import_requests WHERE user_id = $1 AND request_id = $2",
    )
    .bind(user_id)
    .bind(request.id)
    .fetch_optional(&mut **tx)
    .await
    .context("collection import request")?;
    Ok(earlier.map(|(hash, added, updated, removed, cards)| {
        if hash != request.body_sha256 {
            return ImportOutcome::IdReused;
        }
        let count = |n: i32| u32::try_from(n).unwrap_or(0);
        ImportOutcome::Repeated(Applied {
            added: count(added),
            updated: count(updated),
            removed: count(removed),
            cards,
        })
    }))
}

/// Records what the import under `request` did, so the same import sent again answers it.
async fn record_import(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    request: &ImportRequest,
    applied: Applied,
) -> Result<(), anyhow::Error> {
    let stored = |n: u32| i32::try_from(n).unwrap_or(i32::MAX);
    query(
        "INSERT INTO collection_import_requests \
             (user_id, request_id, body_sha256, added, updated, removed, cards) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(user_id)
    .bind(request.id)
    .bind(request.body_sha256.as_slice())
    .bind(stored(applied.added))
    .bind(stored(applied.updated))
    .bind(stored(applied.removed))
    .bind(applied.cards)
    .execute(&mut **tx)
    .await
    .context("collection import request record")?;
    Ok(())
}

/// A replace's removal: every row of the user's the import doesn't hold.
const REMOVE_MISSING: &str = "DELETE FROM collection_cards cc WHERE cc.user_id = $1 AND NOT EXISTS ( \
         SELECT 1 FROM UNNEST($2::uuid[], $3::text[], $4::text[], $5::text[]) \
             AS i(id, finish, condition, language) \
         WHERE cc.scryfall_data_id = i.id AND cc.finish = i.finish \
           AND cc.condition IS NOT DISTINCT FROM i.condition \
           AND cc.language IS NOT DISTINCT FROM i.language)";

/// The import's write, one row per key; `$12` is whether it replaces. A replace sets a row to
/// the import's; an add sums copies (held at 9999), joins tags without regard to case (the row's
/// first, held at 20), keeps a price already there and the earlier date added. Returns whether
/// each row was new.
const WRITE: &str = "INSERT INTO collection_cards AS cc \
         (user_id, scryfall_data_id, oracle_id, finish, condition, language, quantity, \
          tags, purchase_price, purchase_currency, added_at) \
     SELECT $1, i.id, i.oracle_id, i.finish, i.condition, i.language, i.quantity, \
            CASE WHEN i.tags = '' THEN '{}'::text[] ELSE string_to_array(i.tags, chr(31)) END, \
            i.cents::numeric / 100, i.currency, i.added_at \
     FROM UNNEST($2::uuid[], $3::uuid[], $4::text[], $5::text[], $6::text[], $7::int[], \
                 $8::text[], $9::bigint[], $10::text[], $11::timestamptz[]) \
         AS i(id, oracle_id, finish, condition, language, quantity, tags, cents, currency, added_at) \
     ON CONFLICT (user_id, scryfall_data_id, finish, condition, language) DO UPDATE SET \
         quantity = CASE WHEN $12 THEN EXCLUDED.quantity \
             ELSE LEAST(cc.quantity + EXCLUDED.quantity, 9999) END, \
         tags = CASE WHEN $12 THEN EXCLUDED.tags ELSE COALESCE( \
             (SELECT (array_agg(tag ORDER BY ord))[1:20] FROM ( \
                 SELECT DISTINCT ON (lower(t.tag)) t.tag, t.ord \
                 FROM unnest(cc.tags || EXCLUDED.tags) WITH ORDINALITY AS t(tag, ord) \
                 ORDER BY lower(t.tag), t.ord) AS kept), '{}') END, \
         purchase_price = CASE WHEN $12 THEN EXCLUDED.purchase_price \
             ELSE COALESCE(cc.purchase_price, EXCLUDED.purchase_price) END, \
         purchase_currency = CASE WHEN $12 OR cc.purchase_price IS NULL \
             THEN EXCLUDED.purchase_currency ELSE cc.purchase_currency END, \
         added_at = CASE WHEN $12 THEN EXCLUDED.added_at \
             ELSE LEAST(cc.added_at, EXCLUDED.added_at) END, \
         updated_at = now() \
     RETURNING (xmax = 0)";

/// The import's rows as one array per column, for `UNNEST`.
struct ImportArrays<'cards> {
    ids: Vec<Uuid>,
    oracle_ids: Vec<Uuid>,
    finishes: Vec<&'static str>,
    conditions: Vec<Option<&'static str>>,
    languages: Vec<Option<&'static str>>,
    quantities: Vec<i32>,
    /// Each row's tags joined by [`TAG_SEPARATOR`].
    tags: Vec<String>,
    cents: Vec<Option<i64>>,
    currencies: Vec<Option<&'cards str>>,
    added_ats: Vec<Option<DateTime<Utc>>>,
}

impl<'cards> ImportArrays<'cards> {
    fn of(cards: &'cards [CollectionCard]) -> Self {
        let separator = TAG_SEPARATOR.to_string();
        Self {
            ids: cards.iter().map(|c| c.printing.id).collect(),
            oracle_ids: cards.iter().map(|c| c.printing.oracle_id).collect(),
            finishes: cards.iter().map(|c| c.finish.as_str()).collect(),
            conditions: cards
                .iter()
                .map(|c| c.condition.map(CardCondition::as_str))
                .collect(),
            languages: cards
                .iter()
                .map(|c| c.language.map(CardLanguage::as_str))
                .collect(),
            quantities: cards.iter().map(|c| c.quantity.get()).collect(),
            tags: cards
                .iter()
                .map(|c| {
                    let list: Vec<&str> = c
                        .tags
                        .as_slice()
                        .iter()
                        .map(CollectionTag::as_str)
                        .collect();
                    list.join(&separator)
                })
                .collect(),
            cents: cards
                .iter()
                .map(|c| c.purchase_price.map(PurchasePrice::cents))
                .collect(),
            currencies: cards
                .iter()
                .map(|c| c.purchase_currency.as_ref().map(PurchaseCurrency::as_str))
                .collect(),
            added_ats: cards.iter().map(|c| c.added_at).collect(),
        }
    }
}
