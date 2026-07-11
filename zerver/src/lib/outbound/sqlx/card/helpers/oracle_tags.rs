//! Oracle Tags ingest.
//!
//! Full-replaces the `otags` catalog and the `scryfall`-sourced rows of
//! `card_otags` from a fresh Oracle Tags pull, all in one transaction. Rows with
//! a non-`scryfall` `source` (e.g. heuristic backfill in a later phase) are left
//! untouched. See `context/plans/otags/`.

use crate::inbound::external::scryfall::oracle_tag::OracleTag;
use anyhow::Context;
use sqlx::{PgPool, QueryBuilder};

/// Catalog rows per INSERT batch (6 bind params/row, well under Postgres' 65535).
const OTAG_CATALOG_BATCH: usize = 4_000;
/// Correlation rows per INSERT batch (3 bind params/row).
const CARD_OTAG_BATCH: usize = 20_000;

/// Inverts tag -> `[card]` into flat card -> otag correlations, dropping any
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

/// Replaces the otag catalog and the scryfall-sourced card correlations from
/// `tags`. Returns `(catalog_rows, correlation_rows)`.
pub async fn sync_oracle_tags(pool: &PgPool, tags: &[OracleTag]) -> anyhow::Result<(u32, u32)> {
    let correlations = flatten_correlations(tags);

    let mut tx = pool.begin().await?;

    // Catalog: full replace.
    sqlx::query("DELETE FROM otags")
        .execute(&mut *tx)
        .await
        .context("failed to clear otags catalog")?;

    for chunk in tags.chunks(OTAG_CATALOG_BATCH) {
        let mut qb =
            QueryBuilder::new("INSERT INTO otags (id, slug, label, description, parent_ids, aliases) ");
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
            .context("failed to insert otags catalog batch")?;
    }

    // Correlations: replace only the scryfall-sourced rows, preserving other sources.
    sqlx::query("DELETE FROM card_otags WHERE source = 'scryfall'")
        .execute(&mut *tx)
        .await
        .context("failed to clear scryfall card_otags")?;

    for chunk in correlations.chunks(CARD_OTAG_BATCH) {
        let mut qb = QueryBuilder::new("INSERT INTO card_otags (oracle_id, otag, source) ");
        qb.push_values(chunk, |mut b, (oracle_id, otag)| {
            b.push_bind(*oracle_id)
                .push_bind(*otag)
                .push_bind("scryfall");
        });
        qb.push(" ON CONFLICT (oracle_id, otag) DO NOTHING");
        qb.build()
            .execute(&mut *tx)
            .await
            .context("failed to insert card_otags batch")?;
    }

    tx.commit().await?;
    Ok((tags.len() as u32, correlations.len() as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbound::external::scryfall::oracle_tag::Tagging;
    use uuid::Uuid;

    fn tag(slug: &str, oracle_ids: &[Option<Uuid>]) -> OracleTag {
        OracleTag {
            id: Uuid::from_u128(0),
            slug: slug.to_string(),
            label: slug.to_string(),
            description: None,
            parent_ids: vec![],
            aliases: vec![],
            taggings: oracle_ids.iter().map(|&oracle_id| Tagging { oracle_id }).collect(),
        }
    }

    #[test]
    fn flatten_emits_one_row_per_tagging() {
        let a = Uuid::from_u128(0xA);
        let b = Uuid::from_u128(0xB);
        let tags = vec![tag("removal", &[Some(a), Some(b)]), tag("ramp", &[Some(a)])];
        let rows = flatten_correlations(&tags);
        assert_eq!(
            rows,
            vec![(a, "removal"), (b, "removal"), (a, "ramp")]
        );
    }

    #[test]
    fn flatten_skips_taggings_without_oracle_id() {
        let a = Uuid::from_u128(0xA);
        let tags = vec![tag("removal", &[Some(a), None, None])];
        let rows = flatten_correlations(&tags);
        assert_eq!(rows, vec![(a, "removal")]);
    }

    #[test]
    fn flatten_empty_is_empty() {
        assert!(flatten_correlations(&[]).is_empty());
        assert!(flatten_correlations(&[tag("x", &[])]).is_empty());
    }
}
