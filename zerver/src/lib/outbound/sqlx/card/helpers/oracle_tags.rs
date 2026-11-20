//! Oracle Tags ingest.
//!
//! Full-replaces the `oracle_tags` catalog and the `scryfall`-sourced rows of
//! `card_oracle_tags` from a fresh Oracle Tags pull, all in one transaction. Rows
//! with a non-`scryfall` `source` are left untouched. A pull that is empty or under
//! half the stored rows is refused before anything is deleted.

use anyhow::Context;
use sqlx::{PgConnection, QueryBuilder};

use crate::inbound::external::scryfall::oracle_tag::OracleTag;

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

/// How many tags and card tags a pull carries, or the database holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagCounts {
    /// Rows in the tag catalog.
    pub tags: u64,
    /// Scryfall-sourced card tags.
    pub card_tags: u64,
}

/// A pull the sync refuses before it replaces anything, so the stored tags stay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RefusedPullError {
    /// The pull carries no tags or no card tags.
    #[error(
        "the oracle tags pull is empty ({} tags, {} card tags), so the stored {} tags and {} card tags stay",
        pulled.tags, pulled.card_tags, stored.tags, stored.card_tags
    )]
    Empty {
        /// What the pull carries.
        pulled: TagCounts,
        /// What the database holds.
        stored: TagCounts,
    },
    /// The pull carries under half the stored tags or card tags.
    #[error(
        "the oracle tags pull has {} tags and {} card tags, under half the stored {} and {}, so the stored rows stay",
        pulled.tags, pulled.card_tags, stored.tags, stored.card_tags
    )]
    Shrunk {
        /// What the pull carries.
        pulled: TagCounts,
        /// What the database holds.
        stored: TagCounts,
    },
}

/// Accepts a pull unless it is empty or carries under half the stored tags or card tags.
/// Any non-empty pull into an empty catalog is accepted.
///
/// # Errors
///
/// Returns [`RefusedPullError`] naming both counts when the pull is refused.
pub fn check_pull(pulled: TagCounts, stored: TagCounts) -> Result<(), RefusedPullError> {
    if pulled.tags == 0 || pulled.card_tags == 0 {
        return Err(RefusedPullError::Empty { pulled, stored });
    }
    let under_half = |pulled: u64, stored: u64| pulled.saturating_mul(2) < stored;
    if under_half(pulled.tags, stored.tags) || under_half(pulled.card_tags, stored.card_tags) {
        return Err(RefusedPullError::Shrunk { pulled, stored });
    }
    Ok(())
}

/// What the database holds now, read inside the sync's transaction.
async fn stored_counts(tx: &mut PgConnection) -> anyhow::Result<TagCounts> {
    let (tags, card_tags): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM oracle_tags), \
                (SELECT count(*) FROM card_oracle_tags WHERE source = 'scryfall')",
    )
    .fetch_one(&mut *tx)
    .await
    .context("failed to count the stored oracle tags")?;
    Ok(TagCounts {
        tags: u64::try_from(tags).unwrap_or_default(),
        card_tags: u64::try_from(card_tags).unwrap_or_default(),
    })
}

/// Replaces the `oracle_tag` catalog and the scryfall-sourced card correlations
/// from `tags`, inside the caller's transaction `tx`. Returns `(catalog_rows, correlation_rows)`.
/// A pull [`check_pull`] refuses changes nothing.
///
/// # Errors
///
/// Returns [`RefusedPullError`] when the pull is empty or under half the stored rows, and
/// an error if any of its statements fails.
pub async fn sync_oracle_tags(
    tx: &mut PgConnection,
    tags: &[OracleTag],
) -> anyhow::Result<(usize, usize)> {
    let correlations = flatten_correlations(tags);
    let pulled = TagCounts {
        tags: u64::try_from(tags.len()).unwrap_or(u64::MAX),
        card_tags: u64::try_from(correlations.len()).unwrap_or(u64::MAX),
    };
    check_pull(pulled, stored_counts(&mut *tx).await?)?;

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

    fn counts(tags: u64, card_tags: u64) -> TagCounts {
        TagCounts { tags, card_tags }
    }

    #[test]
    fn an_empty_pull_is_refused_even_into_an_empty_catalog() {
        let night = counts(4_606, 241_000);
        assert!(matches!(
            check_pull(counts(0, 0), night),
            Err(RefusedPullError::Empty { .. })
        ));
        assert!(matches!(
            check_pull(counts(4_606, 0), night),
            Err(RefusedPullError::Empty { .. })
        ));
        assert!(matches!(
            check_pull(counts(0, 0), counts(0, 0)),
            Err(RefusedPullError::Empty { .. })
        ));
    }

    #[test]
    fn a_pull_under_half_is_refused_and_half_or_more_goes_through() {
        let stored = counts(4_606, 241_000);
        assert!(matches!(
            check_pull(counts(2_302, 241_000), stored),
            Err(RefusedPullError::Shrunk { .. })
        ));
        assert!(matches!(
            check_pull(counts(4_606, 120_499), stored),
            Err(RefusedPullError::Shrunk { .. })
        ));
        assert_eq!(check_pull(counts(2_303, 120_500), stored), Ok(()));
        assert_eq!(check_pull(counts(4_590, 240_100), stored), Ok(()));
        assert_eq!(check_pull(counts(9_000, 500_000), stored), Ok(()));
    }

    #[test]
    fn a_first_pull_into_an_empty_catalog_goes_through() {
        assert_eq!(check_pull(counts(3, 7), counts(0, 0)), Ok(()));
    }

    #[test]
    fn a_refusal_names_both_counts() {
        let refused = check_pull(counts(0, 0), counts(4_606, 241_000));
        let message = refused.map_err(|e| e.to_string()).err().unwrap_or_default();
        assert!(message.contains("0 tags, 0 card tags"), "{message}");
        assert!(
            message.contains("4606 tags and 241000 card tags stay"),
            "{message}"
        );
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
