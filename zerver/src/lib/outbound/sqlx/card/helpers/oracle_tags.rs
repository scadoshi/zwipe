//! Oracle Tags ingest.
//!
//! Full-replaces the `oracle_tags` catalog and the `scryfall`-sourced rows of
//! `card_oracle_tags` from a fresh Oracle Tags pull, all in one transaction. Rows
//! with a non-`scryfall` `source` are left untouched. A pull that is empty or under
//! [`TAG_FLOOR_PCT`] percent of the stored rows is refused before anything is deleted.

use anyhow::Context;
use sqlx::{PgConnection, QueryBuilder};

use crate::domain::card::models::scryfall_pull::{self, ScryfallLoad, TAG_FLOOR_PCT};
use crate::inbound::external::scryfall::oracle_tag::OracleTag;
use crate::outbound::sqlx::card::helpers::scryfall_loads;

/// Catalog rows per INSERT batch (6 bind params/row, well under Postgres' 65535).
const CATALOG_BATCH: usize = 4_000;
/// Correlation rows per INSERT batch (3 bind params/row).
const CORRELATION_BATCH: usize = 20_000;

/// Inverts tag -> `[card]` into flat card -> `oracle_tag` correlations, dropping any
/// tagging without an oracle id. Borrows each slug from `tags`.
fn flatten_correlations(tags: &[OracleTag]) -> Vec<(uuid::Uuid, &str)> {
    let mut correlations = Vec::new();
    for tag in tags {
        for tagging in &tag.taggings {
            if let Some(oracle_id) = tagging.oracle_id {
                correlations.push((oracle_id, tag.slug.as_str()));
            }
        }
    }
    correlations
}

/// What the database holds now, tags and scryfall card tags, read inside the sync's
/// transaction.
async fn stored_counts(tx: &mut PgConnection) -> anyhow::Result<(u64, u64)> {
    let (tags, card_tags): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM oracle_tags), \
                (SELECT count(*) FROM card_oracle_tags WHERE source = 'scryfall')",
    )
    .fetch_one(&mut *tx)
    .await
    .context("failed to count the stored oracle tags")?;
    Ok((
        u64::try_from(tags).unwrap_or_default(),
        u64::try_from(card_tags).unwrap_or_default(),
    ))
}

/// Refuses a pull of `tags` tags and `card_tags` card tags with none, or under
/// [`TAG_FLOOR_PCT`] percent of what the database holds.
async fn check_pulled(tx: &mut PgConnection, tags: usize, card_tags: usize) -> anyhow::Result<()> {
    let (stored_tags, stored_card_tags) = stored_counts(&mut *tx).await?;
    let count = |n: usize| u64::try_from(n).unwrap_or(u64::MAX);
    scryfall_pull::check_floor("oracle tags", count(tags), stored_tags, TAG_FLOOR_PCT)?;
    scryfall_pull::check_floor(
        "card tags",
        count(card_tags),
        stored_card_tags,
        TAG_FLOOR_PCT,
    )?;
    Ok(())
}

/// Replaces the `oracle_tag` catalog and the scryfall-sourced card correlations
/// from `tags`, inside the caller's transaction `tx`, and records `load` when given. Returns
/// `(catalog_rows, correlation_rows)`. A pull with no tags or card tags, or under
/// [`TAG_FLOOR_PCT`] percent of the stored ones, changes nothing.
///
/// # Errors
///
/// Returns [`scryfall_pull::RefusedPull`] when the pull is refused, and an error if any of its statements
/// fails.
pub async fn sync_oracle_tags(
    tx: &mut PgConnection,
    tags: &[OracleTag],
    load: Option<&ScryfallLoad>,
) -> anyhow::Result<(usize, usize)> {
    let correlations = flatten_correlations(tags);
    check_pulled(&mut *tx, tags.len(), correlations.len()).await?;
    if let Some(load) = load {
        scryfall_loads::record_load(&mut *tx, load).await?;
    }

    // Catalog: full replace.
    sqlx::query("DELETE FROM oracle_tags")
        .execute(&mut *tx)
        .await
        .context("failed to clear oracle_tags catalog")?;

    for chunk in tags.chunks(CATALOG_BATCH) {
        let mut qb = QueryBuilder::new(
            "INSERT INTO oracle_tags (id, slug, label, description, parent_ids, aliases) ",
        );
        qb.push_values(chunk, |mut b, tag| {
            b.push_bind(tag.id)
                .push_bind(tag.slug.clone())
                .push_bind(tag.label.clone())
                .push_bind(tag.description.clone())
                .push_bind(tag.parent_ids.clone())
                .push_bind(tag.aliases.clone());
        });
        qb.push(" ON CONFLICT (id) DO NOTHING");
        qb.build()
            .execute(&mut *tx)
            .await
            .context("failed to insert oracle_tags catalog batch")?;
    }

    // Overlay our authored descriptions onto the fresh catalog, in the same
    // transaction so they're never briefly missing. Ours wins: replaces
    // Scryfall's where we have one, fills the blanks otherwise. See
    // `oracle_tag_descriptions`.
    let (desc_slugs, descriptions) = super::oracle_tag_descriptions::description_pairs();
    let mut unknown_count = 0usize;
    if !desc_slugs.is_empty() {
        sqlx::query(
            "UPDATE oracle_tags AS ot SET description = d.description \
             FROM unnest($1::text[], $2::text[]) AS d(slug, description) \
             WHERE ot.slug = d.slug",
        )
        .bind(&desc_slugs)
        .bind(&descriptions)
        .execute(&mut *tx)
        .await
        .context("failed to overlay authored oracle-tag descriptions")?;

        // Non-fatal typo guard: an authored slug the fresh catalog lacks would
        // silently match nothing, so surface it in the sync log.
        let unknown: Vec<String> = sqlx::query_scalar(
            "SELECT s FROM unnest($1::text[]) AS s \
             WHERE NOT EXISTS (SELECT 1 FROM oracle_tags WHERE slug = s)",
        )
        .bind(&desc_slugs)
        .fetch_all(&mut *tx)
        .await
        .unwrap_or_default();
        if !unknown.is_empty() {
            unknown_count = unknown.len();
            tracing::warn!(
                "ORACLE_TAG_DESCRIPTIONS references unknown oracle-tag slugs: {unknown:?}"
            );
        }
    }

    // Coverage line: the opposite gap to the typo guard above. Three numbers,
    // two goals: `blank` is the user-visible gap (no description from any
    // source), `authored` tracks the replace-Scryfall bulk-authoring goal
    // (the rest are Scryfall's own copy). Visible in every nightly log so
    // coverage drift is impossible to miss.
    let (covered, blank): (i64, i64) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE description IS NOT NULL AND description <> ''), \
                count(*) FILTER (WHERE description IS NULL OR description = '') \
         FROM oracle_tags",
    )
    .fetch_one(&mut *tx)
    .await
    .context("failed to count oracle-tag description coverage")?;
    let authored = desc_slugs.len().saturating_sub(unknown_count);
    tracing::info!(
        "oracle tag descriptions: {covered}/{} have one ({authored} authored, {blank} blank)",
        covered.saturating_add(blank)
    );

    // Correlations: replace only the scryfall-sourced rows, preserving other sources.
    sqlx::query("DELETE FROM card_oracle_tags WHERE source = 'scryfall'")
        .execute(&mut *tx)
        .await
        .context("failed to clear scryfall card_oracle_tags")?;

    for chunk in correlations.chunks(CORRELATION_BATCH) {
        let mut qb =
            QueryBuilder::new("INSERT INTO card_oracle_tags (oracle_id, oracle_tag, source) ");
        qb.push_values(chunk, |mut b, (oracle_id, oracle_tag)| {
            b.push_bind(*oracle_id)
                .push_bind(*oracle_tag)
                .push_bind("scryfall");
        });
        qb.push(" ON CONFLICT (oracle_id, oracle_tag) DO NOTHING");
        qb.build()
            .execute(&mut *tx)
            .await
            .context("failed to insert card_oracle_tags batch")?;
    }

    Ok((tags.len(), correlations.len()))
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::inbound::external::scryfall::oracle_tag::Tagging;

    fn tag(slug: &str, oracle_ids: &[Option<Uuid>]) -> OracleTag {
        OracleTag {
            id: Uuid::from_u128(0),
            slug: slug.to_string(),
            label: slug.to_string(),
            description: None,
            parent_ids: vec![],
            aliases: vec![],
            taggings: oracle_ids
                .iter()
                .map(|&oracle_id| Tagging { oracle_id })
                .collect(),
        }
    }

    #[test]
    fn flatten_emits_one_row_per_tagging() {
        let a = Uuid::from_u128(0xa);
        let b = Uuid::from_u128(0xb);
        let tags = vec![tag("removal", &[Some(a), Some(b)]), tag("ramp", &[Some(a)])];
        let rows = flatten_correlations(&tags);
        assert_eq!(rows, vec![(a, "removal"), (b, "removal"), (a, "ramp")]);
    }

    #[test]
    fn flatten_skips_taggings_without_oracle_id() {
        let a = Uuid::from_u128(0xa);
        let tags = vec![tag("removal", &[Some(a), None, None])];
        let rows = flatten_correlations(&tags);
        assert_eq!(rows, vec![(a, "removal")]);
    }

    #[test]
    fn flatten_empty_is_empty() {
        assert_eq!(flatten_correlations(&[]), Vec::<(uuid::Uuid, &str)>::new());
        assert_eq!(
            flatten_correlations(&[tag("x", &[])]),
            Vec::<(uuid::Uuid, &str)>::new()
        );
    }
}
