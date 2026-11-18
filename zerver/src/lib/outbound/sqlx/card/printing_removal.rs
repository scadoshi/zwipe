//! The SQL behind removing the printings Scryfall deleted: the stored printings, the survivors
//! a removed one moves to, what references each, and the batched remap and delete. Every delete
//! checks that nothing still references the printing, so a deck never loses a card, whatever the
//! plan says.

use std::collections::HashMap;

use sqlx::{PgPool, Postgres, Transaction, query, query_as, query_scalar};
use uuid::Uuid;

use super::IMPORTABLE_PRINTING;
use crate::domain::card::models::printing_removal::{
    Printing, References, RemovalCounts, RemovalPlan, Survivors,
};

/// Removals applied per transaction: small enough that a batch's locks stay brief, large enough
/// that a backlog of thousands takes a few dozen round trips.
const BATCH: usize = 100;

/// The printings import and Quick add land for a card with no named printing, over `sd`
/// (`scryfall_data`): English, paper, regular size, not a promo, sold in nonfoil, never an Art
/// Series, token or memorabilia printing, with a USD price. A USD price under a tenth of the
/// printing's EUR price reads as a bad figure (#47) and that printing is skipped; with no EUR price
/// the USD price stands. CASE keeps the casts behind the checks.
pub(super) fn cheapest_regular_filter() -> String {
    format!(
        "{IMPORTABLE_PRINTING} \
         AND sd.lang = 'en' AND NOT sd.digital AND NOT sd.oversized \
         AND NOT sd.promo AND 'nonfoil' = ANY(sd.finishes) \
         AND sd.prices->>'usd' ~ '^[0-9]+(\\.[0-9]+)?$' \
         AND CASE WHEN sd.prices->>'eur' ~ '^[0-9]+(\\.[0-9]+)?$' \
             THEN (sd.prices->>'usd')::numeric >= (sd.prices->>'eur')::numeric / 10 \
             ELSE true END"
    )
}

/// Cheapest first, after a `DISTINCT ON (sd.oracle_id)` key: the lowest USD price, then the newest
/// release, then the id so a tie always picks the same printing.
pub(super) const CHEAPEST_REGULAR_ORDER: &str =
    "(sd.prices->>'usd')::numeric, sd.released_at DESC, sd.id";

/// The printing `latest_cards` shows for a card, after a `DISTINCT ON (sd.oracle_id)` key: the
/// same keys as the view (migration 20260922120000), then the id.
const PREFERRED_ORDER: &str = "sd.digital, sd.promo, sd.oversized, \
     COALESCE(sd.content_warning, false), (sd.lang IS DISTINCT FROM 'en'), \
     (COALESCE(sd.security_stamp, '') = 'triangle' OR sd.set IN (SELECT code FROM oou_sets)), \
     (sd.layout = 'reversible_card'), sd.released_at DESC, sd.id";

/// One printing as the plan reads it: id, oracle id, name, set, collector number.
type PrintingRow = (Uuid, Option<Uuid>, String, String, String);

fn printing((id, oracle_id, name, set, collector_number): PrintingRow) -> Printing {
    Printing {
        id,
        oracle_id,
        name,
        set,
        collector_number,
    }
}

/// Every stored printing.
pub(super) async fn stored(pool: &PgPool) -> Result<Vec<Printing>, sqlx::Error> {
    let rows: Vec<PrintingRow> =
        query_as("SELECT id, oracle_id, name, set, collector_number FROM scryfall_data")
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(printing).collect())
}

/// One printing per card of `oracle_ids`, never one of `removed`, picked by `filter` and `order`.
async fn pick(
    pool: &PgPool,
    oracle_ids: &[Uuid],
    removed: &[Uuid],
    filter: &str,
    order: &str,
) -> Result<HashMap<Uuid, Printing>, sqlx::Error> {
    let sql = format!(
        "SELECT DISTINCT ON (sd.oracle_id) sd.id, sd.oracle_id, sd.name, sd.set, sd.collector_number \
         FROM scryfall_data sd JOIN card_profiles cp ON sd.id = cp.scryfall_data_id \
         WHERE sd.oracle_id = ANY($1) AND sd.id <> ALL($2) AND {filter} \
         ORDER BY sd.oracle_id, {order}"
    );
    // Built from string constants only; every value is bound.
    let rows: Vec<PrintingRow> = query_as(sqlx::AssertSqlSafe(sql))
        .bind(oracle_ids)
        .bind(removed)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| row.1.map(|oracle_id| (oracle_id, printing(row))))
        .collect())
}

/// The cheapest regular and the preferred surviving printing of each of `oracle_ids`.
pub(super) async fn survivors(
    pool: &PgPool,
    oracle_ids: &[Uuid],
    removed: &[Uuid],
) -> Result<Survivors, sqlx::Error> {
    if oracle_ids.is_empty() {
        return Ok(Survivors::default());
    }
    let cheapest = pick(
        pool,
        oracle_ids,
        removed,
        &cheapest_regular_filter(),
        CHEAPEST_REGULAR_ORDER,
    )
    .await?;
    let preferred = pick(pool, oracle_ids, removed, "true", PREFERRED_ORDER).await?;
    Ok(Survivors {
        cheapest,
        preferred,
    })
}

/// The deck card rows, command zone slots and collection rows that point at each of `ids`; an
/// id nothing references is left out.
pub(super) async fn references(
    pool: &PgPool,
    ids: &[Uuid],
) -> Result<HashMap<Uuid, References>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows: Vec<(Uuid, i64, i64, i64)> = query_as(
        "SELECT r.id, \
             (SELECT count(*) FROM deck_cards dc WHERE dc.scryfall_data_id = r.id), \
             (SELECT COALESCE(SUM( \
                 (d.commander_id IS NOT DISTINCT FROM r.id)::int \
               + (d.partner_commander_id IS NOT DISTINCT FROM r.id)::int \
               + (d.background_id IS NOT DISTINCT FROM r.id)::int \
               + (d.signature_spell_id IS NOT DISTINCT FROM r.id)::int), 0) \
              FROM decks d \
              WHERE r.id IN (d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id)), \
             (SELECT count(*) FROM collection_cards cc WHERE cc.scryfall_data_id = r.id) \
         FROM unnest($1::uuid[]) AS r(id)",
    )
    .bind(ids)
    .fetch_all(pool)
    .await?;
    let count = |n: i64| u32::try_from(n).unwrap_or(u32::MAX);
    Ok(rows
        .into_iter()
        .map(|(id, deck_cards, slots, collection_cards)| {
            (
                id,
                References {
                    deck_cards: count(deck_cards),
                    slots: count(slots),
                    collection_cards: count(collection_cards),
                },
            )
        })
        .filter(|(_, held)| held.any())
        .collect())
}

/// Applies `plan` a batch at a time, each batch one transaction; a plan with nothing to remove
/// still makes one pass, which moves any stale reference an older client wrote since the last run.
pub(super) async fn apply(pool: &PgPool, plan: &RemovalPlan) -> Result<RemovalCounts, sqlx::Error> {
    let mut total = RemovalCounts::default();
    let batches: Vec<&[_]> = if plan.removals.is_empty() {
        vec![&[]]
    } else {
        plan.removals.chunks(BATCH).collect()
    };
    for batch in batches {
        let mut tx = pool.begin().await?;
        let counts = apply_batch(&mut tx, batch).await?;
        tx.commit().await?;
        total.removed = total.removed.saturating_add(counts.removed);
        total.remapped_deck_cards = total
            .remapped_deck_cards
            .saturating_add(counts.remapped_deck_cards);
        total.remapped_slots = total.remapped_slots.saturating_add(counts.remapped_slots);
        total.remapped_collection_cards = total
            .remapped_collection_cards
            .saturating_add(counts.remapped_collection_cards);
    }
    Ok(total)
}

/// One batch: log the removals, collapse older log rows whose replacement this batch removes,
/// move every deck reference to a logged printing onto its live replacement, delete what nothing
/// references any more, and drop the log rows of printings that stayed.
async fn apply_batch(
    tx: &mut Transaction<'_, Postgres>,
    batch: &[crate::domain::card::models::printing_removal::Removal],
) -> Result<RemovalCounts, sqlx::Error> {
    let ids: Vec<Uuid> = batch.iter().map(|r| r.printing.id).collect();
    let oracles: Vec<Option<Uuid>> = batch.iter().map(|r| r.printing.oracle_id).collect();
    let replacements: Vec<Option<Uuid>> = batch
        .iter()
        .map(|r| r.replacement.as_ref().map(|p| p.id))
        .collect();
    query(
        "INSERT INTO removed_printings (id, oracle_id, replacement_id) \
         SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::uuid[]) \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(&ids)
    .bind(&oracles)
    .bind(&replacements)
    .execute(&mut **tx)
    .await?;
    // An older removal whose replacement this batch removes points on to the new replacement.
    query(
        "UPDATE removed_printings old SET replacement_id = new.replacement_id \
         FROM removed_printings new \
         WHERE old.replacement_id = new.id AND new.replacement_id IS NOT NULL",
    )
    .execute(&mut **tx)
    .await?;

    // Only onto a replacement that exists, so a stale reference to a long-gone chain never fails
    // the foreign key.
    let live = "rp.replacement_id IS NOT NULL \
                AND EXISTS (SELECT 1 FROM scryfall_data live WHERE live.id = rp.replacement_id)";
    let remapped_deck_cards = query(sqlx::AssertSqlSafe(format!(
        "UPDATE deck_cards dc SET scryfall_data_id = rp.replacement_id \
         FROM removed_printings rp WHERE dc.scryfall_data_id = rp.id AND {live}"
    )))
    .execute(&mut **tx)
    .await?
    .rows_affected();
    let mut remapped_slots = 0u64;
    for slot in [
        "commander_id",
        "partner_commander_id",
        "background_id",
        "signature_spell_id",
    ] {
        // Built from string constants only; every value is bound.
        let moved = query(sqlx::AssertSqlSafe(format!(
            "UPDATE decks d SET {slot} = rp.replacement_id \
             FROM removed_printings rp WHERE d.{slot} = rp.id AND {live}"
        )))
        .execute(&mut **tx)
        .await?
        .rows_affected();
        remapped_slots = remapped_slots.saturating_add(moved);
    }

    let remapped_collection_cards = remap_collection(tx, live).await?;

    let unreferenced = |column: &str| {
        format!(
            "NOT EXISTS (SELECT 1 FROM deck_cards dc WHERE dc.scryfall_data_id = {column}) \
             AND NOT EXISTS (SELECT 1 FROM collection_cards cc WHERE cc.scryfall_data_id = {column}) \
             AND NOT EXISTS (SELECT 1 FROM decks d WHERE {column} IN \
                 (d.commander_id, d.partner_commander_id, d.background_id, d.signature_spell_id))"
        )
    };
    query(sqlx::AssertSqlSafe(format!(
        "DELETE FROM card_profiles cp WHERE cp.scryfall_data_id = ANY($1) AND {}",
        unreferenced("cp.scryfall_data_id")
    )))
    .bind(&ids)
    .execute(&mut **tx)
    .await?;
    let removed = query(sqlx::AssertSqlSafe(format!(
        "DELETE FROM scryfall_data sd WHERE sd.id = ANY($1) \
         AND NOT EXISTS (SELECT 1 FROM card_profiles cp WHERE cp.scryfall_data_id = sd.id) AND {}",
        unreferenced("sd.id")
    )))
    .bind(&ids)
    .execute(&mut **tx)
    .await?
    .rows_affected();
    // A printing that stayed (a reference appeared since the plan) logs no removal.
    query(
        "DELETE FROM removed_printings rp WHERE rp.id = ANY($1) \
         AND EXISTS (SELECT 1 FROM scryfall_data sd WHERE sd.id = rp.id)",
    )
    .bind(&ids)
    .execute(&mut **tx)
    .await?;
    Ok(RemovalCounts {
        removed,
        remapped_deck_cards,
        remapped_slots,
        remapped_collection_cards,
    })
}

/// Moves every collection row on a removed printing with a live replacement onto that
/// replacement. Rows that meet on the same printing, finish, condition and language merge: copies
/// add up to the most a row holds, tags join without regard to case (the first spelling and order
/// kept, the row already there first) up to the most a row carries, and the earliest date
/// added stays, with the purchase price of the earliest row that has one. A moved row takes the
/// replacement's oracle id. Returns how many rows moved.
async fn remap_collection(
    tx: &mut Transaction<'_, Postgres>,
    live: &str,
) -> Result<u64, sqlx::Error> {
    // One statement, so a row committed on a removed printing while it runs is either moved or
    // left for the next run, never deleted unmoved. Only rows whose replacement exists (`live`)
    // move; the rest stay on their printing, which counts them as references and keeps it. The
    // replacement's card is a LEFT JOIN, so a gap there fails the insert instead of dropping rows. Built from string constants only; every
    // value comes from the tables.
    let moved: i64 = query_scalar(sqlx::AssertSqlSafe(format!(
        "WITH moved AS ( \
             DELETE FROM collection_cards cc USING removed_printings rp \
             WHERE cc.scryfall_data_id = rp.id AND {live} \
             RETURNING cc.*, rp.replacement_id \
         ), \
         grouped AS ( \
             SELECT m.user_id, m.replacement_id, m.finish, m.condition, m.language, \
                    COALESCE(rep.oracle_id, min(m.oracle_id::text)::uuid) AS oracle_id, \
                    LEAST(sum(m.quantity), 9999)::int AS quantity, \
                    (array_agg(m.purchase_price ORDER BY m.added_at NULLS LAST) \
                        FILTER (WHERE m.purchase_price IS NOT NULL))[1] AS purchase_price, \
                    (array_agg(m.purchase_currency ORDER BY m.added_at NULLS LAST) \
                        FILTER (WHERE m.purchase_price IS NOT NULL))[1] AS purchase_currency, \
                    min(m.added_at) AS added_at \
             FROM moved m LEFT JOIN scryfall_data rep ON rep.id = m.replacement_id \
             GROUP BY m.user_id, m.replacement_id, m.finish, m.condition, m.language, rep.oracle_id \
         ), \
         written AS ( \
         INSERT INTO collection_cards AS cc \
             (user_id, scryfall_data_id, oracle_id, finish, condition, language, quantity, tags, \
              purchase_price, purchase_currency, added_at) \
         SELECT g.user_id, g.replacement_id, g.oracle_id, g.finish, g.condition, g.language, \
                g.quantity, \
                COALESCE((SELECT (array_agg(tag ORDER BY pos))[1:20] FROM ( \
                              SELECT DISTINCT ON (lower(t.tag)) t.tag, row_number() OVER ( \
                                  ORDER BY m.added_at NULLS LAST, m.scryfall_data_id, t.ord) AS pos \
                              FROM moved m, unnest(m.tags) WITH ORDINALITY AS t(tag, ord) \
                              WHERE m.user_id = g.user_id AND m.replacement_id = g.replacement_id \
                                AND m.finish = g.finish \
                                AND m.condition IS NOT DISTINCT FROM g.condition \
                                AND m.language IS NOT DISTINCT FROM g.language \
                              ORDER BY lower(t.tag), pos) AS kept), '{{}}'), \
                g.purchase_price, g.purchase_currency, g.added_at \
         FROM grouped g \
         ON CONFLICT (user_id, scryfall_data_id, finish, condition, language) DO UPDATE SET \
             quantity = LEAST(cc.quantity + EXCLUDED.quantity, 9999), \
             tags = COALESCE((SELECT (array_agg(tag ORDER BY ord))[1:20] FROM ( \
                                  SELECT DISTINCT ON (lower(t.tag)) t.tag, t.ord \
                                  FROM unnest(cc.tags || EXCLUDED.tags) WITH ORDINALITY AS t(tag, ord) \
                                  ORDER BY lower(t.tag), t.ord) AS kept), '{{}}'), \
             purchase_price = COALESCE(cc.purchase_price, EXCLUDED.purchase_price), \
             purchase_currency = CASE WHEN cc.purchase_price IS NULL \
                 THEN EXCLUDED.purchase_currency ELSE cc.purchase_currency END, \
             added_at = LEAST(cc.added_at, EXCLUDED.added_at), \
             updated_at = now() \
         RETURNING 1 \
         ) \
         SELECT count(*) FROM moved"
    )))
    .fetch_one(&mut **tx)
    .await?;
    Ok(u64::try_from(moved).unwrap_or(0))
}
