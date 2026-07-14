//! Derive card categories from Oracle Tags (the retirement of the regex heuristic).
//!
//! For the ~18 `CardRole` variants that map cleanly to an Oracle-tag
//! subtree, plus `Tokens` from Scryfall's `all_parts` token metadata, this computes
//! `card_profiles.card_roles` directly from the community gold standard.
//! See `context/plans/otags/` (Phase 2, Option A).
//!
//! `Pump`, `Stax`, `Protection`, `GraveyardHate` also map to an otag root here, but
//! only as a *supplement*: they stay heuristic-tagged too (`oracle_tag_gaps.rs`,
//! merged in `zervice`), and their union with the otag subtree is what lets those
//! roles group their tags on the card display instead of dumping them in "Other".
//! Protection/GraveyardHate have clean roots; Pump/Stax roots are partial (they
//! capture only `combat-trick` / `tax` slices), so some of their tags still fall
//! through. `Finisher` is dropped.

use anyhow::Context;
use sqlx::PgPool;

/// `CardRole` (serde snake_case) → the Oracle-tag root slug(s) whose
/// subtree defines it. Each root is expanded through `oracle_tags.parent_ids`.
/// The 4 heuristic stragglers are also here as supplements (see module docs);
/// only `Finisher` is absent.
pub const CATEGORY_ROOTS: &[(&str, &[&str])] = &[
    (
        "anthem",
        &[
            "anthem",
            "keyword-anthem",
            "power-boost-to-all",
            "toughness-boost-to-all",
        ],
    ),
    (
        "aggression",
        &[
            "attacking-matters",
            "attacking-matters-self",
            "attacking-matters-any",
            "attack-trigger",
            "gives-haste",
        ],
    ),
    ("blink", &["flicker"]),
    ("burn", &["burn"]),
    // Broad umbrella: everything that nets cards. Draw (below) is the literal-draw
    // subset of this, so the two intentionally overlap.
    ("card_advantage", &["card-advantage", "hand-positive"]),
    ("copy", &["copy", "clone"]),
    ("counterspell", &["counterspell"]),
    (
        "counters",
        &[
            "gains-pp-counters",
            "gives-pp-counters",
            "repeatable-pp-counters",
            "counters-matter",
            "gives-mm-counters",
            "counter-increaser",
            "gains-mm-counters",
        ],
    ),
    ("drain", &["drain-life"]),
    // Literal card-drawing only — the narrow subset of `card_advantage`.
    (
        "draw",
        &[
            "pure-draw",
            "repeatable-pure-draw",
            "burst-draw",
            "draw-engine",
            "impulsive-draw",
            "repeatable-impulsive-draw",
            "long-term-impulsive-draw",
            "force-draw",
        ],
    ),
    (
        "energy",
        &[
            "energy-generator",
            "counter-fuel-energy",
            "energy-increaser",
            "synergy-energy",
        ],
    ),
    ("evasion", &["evasion", "gives-evasion"]),
    ("graveyard_hate", &["hate-graveyard"]),
    ("lifegain", &["lifegain", "lifegain-matters"]),
    ("mill", &["mill"]),
    ("protection", &["protection"]),
    ("pump", &["combat-trick"]),
    ("ramp", &["ramp", "mana-producer"]),
    ("recursion", &["recursion", "reanimate"]),
    ("removal", &["removal", "lockdown"]),
    ("sacrifice", &["sacrifice-outlet"]),
    ("stax", &["tax", "lockdown", "pillowfort"]),
    // Supplements the all_parts-based Tokens membership: nests the token-generation
    // otag family under the Tokens role instead of dumping it in "Other".
    (
        "tokens",
        &[
            "repeatable-token-generator",
            "synergy-token",
            "token-versions-of-cards",
            "token-increaser",
        ],
    ),
    ("tutor", &["tutor"]),
    ("untap", &["untapper"]),
    ("wipe", &["sweeper", "mass-land-denial"]),
];

/// Manual one-off patches: exact oracle-tag slugs whose wording dodges Scryfall's
/// hierarchy but clearly belong to a role. Unlike [`CATEGORY_ROOTS`], these are
/// matched **exactly** — never subtree-expanded — so an entry adds precisely that
/// tag and nothing beneath it. Key = a real `CardRole` (snake_case),
/// checked by unit test; value = real oracle-tag slugs, checked by a non-fatal
/// `zervice` warn. Fed into both derivation and grouping. Grow as you audit;
/// applies on the next `zervice` run.
pub const ROLE_TAG_OVERRIDES: &[(&str, &[&str])] = &[
    (
        "protection",
        &[
            "fog-selective",
            "damage-prevention",
            "phasing",
            "gains-indestructible",
            "regenerates-self",
            "damage-prevention-you",
            "damage-prevention-self",
            "regenerates-other",
            "pseudo-fog",
        ],
    ),
    (
        "ramp",
        &[
            "gives-mana-ability",
            "repeatable-treasures",
            "adds-multiple-mana",
        ],
    ),
    ("aggression", &["gains-haste", "extra-combat-phase"]),
    ("burn", &["pinger"]),
    ("copy", &["conjure-duplicate"]),
    ("counters", &["move-counters", "pseudo-proliferate"]),
    ("drain", &["drain-creature"]),
    ("graveyard_hate", &["hate-graveyard-cast"]),
    ("lifegain", &["lifegain-increaser"]),
    ("mill", &["synergy-mill"]),
    ("pump", &["shade-pump", "firebreathing"]),
    (
        "recursion",
        &[
            "castable-from-graveyard",
            "temporary-reanimation",
            "mass-reanimation",
        ],
    ),
    ("removal", &["swap-removal", "mass-land-denial"]),
    (
        "sacrifice",
        &[
            "synergy-sacrifice",
            "sacrifice-self",
            "synergy-sacrifice-self",
        ],
    ),
    (
        "stax",
        &[
            "mass-land-denial",
            "prevent-activation",
            "prevent-cast",
            "hatebear",
        ],
    ),
    (
        "tokens",
        &[
            "out-of-color-token",
            "unique-token",
            "temporary-token",
            "donate-token",
            "unprinted-token",
            "named-token",
        ],
    ),
    ("tutor", &["booster-tutor", "synergy-tutor"]),
    (
        "untap",
        &[
            "extra-untap",
            "untapper-planeswalker",
            "untapper-equipment",
            "ritual-untap",
        ],
    ),
];

/// Flatten [`ROLE_TAG_OVERRIDES`] into parallel `(role, slug)` arrays for `unnest`.
pub fn override_pairs() -> (Vec<String>, Vec<String>) {
    pairs(ROLE_TAG_OVERRIDES)
}

/// Exact `(role, oracle-tag slug)` pairs to **subtract** from a role after subtree
/// expansion: tags whose Scryfall parent chain drags them under a role's root but
/// which don't belong there. The subtree mechanism only adds, and a tag can sit
/// under several roots at once (multi-parent), so narrowing a root can't remove
/// these cleanly — an explicit exclusion is the only sync-proof lever. Matched
/// **exactly** (no subtree expansion), keyed by a real `CardRole` (unit-test
/// checked); slug side is warn-checked against the live catalog. Applied in both
/// derivation and grouping. Grow as you audit; applies on the next `zervice` run.
pub const ROLE_TAG_EXCLUSIONS: &[(&str, &[&str])] = &[
    // burn-self hangs off burn-creature deep under the removal subtree; it is a
    // self-burn creature, not removal.
    ("removal", &["burn-self"]),
    // twiddle is a direct child of the `untapper` root but also means *tap*
    // (opponent's permanents), so it over-selects for the untap role.
    ("untap", &["twiddle"]),
    // counter-fuel-aesthetic is flavor/counter-fuel, not a +1/+1-counters payoff.
    ("counters", &["counter-fuel-aesthetic"]),
    // repeatable-maps (Maps) is a multi-parent artifact-token/surveil tag; it
    // lands under mill via `surveil` but isn't a mill payoff.
    ("mill", &["repeatable-maps"]),
];

/// Flatten [`ROLE_TAG_EXCLUSIONS`] into parallel `(role, slug)` arrays for `unnest`.
pub fn exclusion_pairs() -> (Vec<String>, Vec<String>) {
    pairs(ROLE_TAG_EXCLUSIONS)
}

/// Flatten a `(role, &[slug])` table into parallel `(role, slug)` arrays.
fn pairs(table: &[(&str, &[&str])]) -> (Vec<String>, Vec<String>) {
    let mut roles = Vec::new();
    let mut slugs = Vec::new();
    for (role, tags) in table {
        for tag in *tags {
            roles.push((*role).to_string());
            slugs.push((*tag).to_string());
        }
    }
    (roles, slugs)
}

/// Non-fatal typo guard: log any slug the `oracle_tags` catalog doesn't carry, so
/// a bad `ROLE_TAG_OVERRIDES` / `ROLE_TAG_EXCLUSIONS` entry surfaces in the zervice
/// log instead of silently matching nothing. `source` names the const for the log.
async fn warn_unknown_slugs(pool: &PgPool, slugs: &[String], source: &str) {
    if slugs.is_empty() {
        return;
    }
    let unknown: Vec<String> = sqlx::query_scalar(
        "SELECT s FROM unnest($1::text[]) AS s
         WHERE NOT EXISTS (SELECT 1 FROM oracle_tags WHERE slug = s)",
    )
    .bind(slugs)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    if !unknown.is_empty() {
        tracing::warn!("{source} references unknown oracle-tag slugs: {unknown:?}");
    }
}

/// Rebuilds `card_profiles.card_roles` from Oracle Tags: expands each
/// category's roots through the tag hierarchy, unions the matching cards, adds
/// `Tokens` for any card whose `all_parts` contains a token component, and writes
/// the sorted category array per printing. Returns rows affected.
///
/// Writes the **otag-derived + Tokens portion only** — the 4 heuristic stragglers
/// are also merged in from `oracle_tag_gaps.rs` at integration (see module docs).
/// Wired into `zervice` as the first step of `derive_card_categories`.
pub async fn derive_categories(pool: &PgPool) -> anyhow::Result<u64> {
    // Flatten the map into two parallel arrays (category, root_slug), passed via unnest.
    let mut categories: Vec<String> = Vec::new();
    let mut root_slugs: Vec<String> = Vec::new();
    for (cat, roots) in CATEGORY_ROOTS {
        for root in *roots {
            categories.push((*cat).to_string());
            root_slugs.push((*root).to_string());
        }
    }
    let (ov_categories, ov_slugs) = override_pairs();
    let (ex_categories, ex_slugs) = exclusion_pairs();
    warn_unknown_slugs(pool, &ov_slugs, "ROLE_TAG_OVERRIDES").await;
    warn_unknown_slugs(pool, &ex_slugs, "ROLE_TAG_EXCLUSIONS").await;

    let result = sqlx::query(
        "WITH RECURSIVE roots(category, root_slug) AS (
             SELECT c, r FROM unnest($1::text[], $2::text[]) AS t(c, r)
         ),
         excl(category, slug) AS (
             SELECT c, s FROM unnest($5::text[], $6::text[]) AS e(c, s)
         ),
         seeded(category, id, slug) AS (
             SELECT r.category, ot.id, ot.slug FROM roots r JOIN oracle_tags ot ON ot.slug = r.root_slug
         ),
         subtree(category, id, slug) AS (
             SELECT category, id, slug FROM seeded
             UNION
             SELECT st.category, c.id, c.slug FROM subtree st JOIN oracle_tags c ON st.id = ANY(c.parent_ids)
         ),
         derived(oracle_id, category) AS (
             SELECT DISTINCT co.oracle_id, s.category FROM card_oracle_tags co JOIN subtree s ON s.slug = co.oracle_tag
               WHERE NOT EXISTS (SELECT 1 FROM excl e WHERE e.category = s.category AND e.slug = co.oracle_tag)
             UNION
             -- exact leaf overrides: matched as-is, no subtree expansion
             SELECT DISTINCT co.oracle_id, o.category
               FROM card_oracle_tags co JOIN unnest($3::text[], $4::text[]) AS o(category, slug) ON o.slug = co.oracle_tag
               WHERE NOT EXISTS (SELECT 1 FROM excl e WHERE e.category = o.category AND e.slug = co.oracle_tag)
             UNION
             SELECT DISTINCT sd.oracle_id, 'tokens' FROM scryfall_data sd
               WHERE sd.all_parts @> '[{\"component\":\"token\"}]'::jsonb AND sd.oracle_id IS NOT NULL
         ),
         agg(oracle_id, cats) AS (
             SELECT oracle_id, jsonb_agg(DISTINCT category ORDER BY category) AS cats FROM derived GROUP BY oracle_id
         )
         UPDATE card_profiles cp
         SET card_roles = COALESCE(a.cats, '[]'::jsonb)
         FROM scryfall_data sd
         LEFT JOIN agg a ON a.oracle_id = sd.oracle_id
         WHERE cp.scryfall_data_id = sd.id",
    )
    .bind(&categories)
    .bind(&root_slugs)
    .bind(&ov_categories)
    .bind(&ov_slugs)
    .bind(&ex_categories)
    .bind(&ex_slugs)
    .execute(pool)
    .await
    .context("failed to derive card_roles from oracle tags")?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zwipe_core::domain::card::card_role::CardRole;

    /// Every mapped category string must round-trip to a real `CardRole`,
    /// so the map can't drift from the enum's snake_case serde.
    #[test]
    fn category_strings_are_valid_variants() {
        for (cat, _) in CATEGORY_ROOTS {
            assert!(
                CardRole::try_from(*cat).is_ok(),
                "'{cat}' is not a CardRole variant",
            );
        }
    }

    /// Every override key must be a real card role too (the slug side is checked
    /// at runtime against the live catalog by `warn_unknown_slugs`).
    #[test]
    fn override_roles_are_valid_variants() {
        for (role, _) in ROLE_TAG_OVERRIDES {
            assert!(
                CardRole::try_from(*role).is_ok(),
                "'{role}' is not a CardRole variant",
            );
        }
    }

    /// Every exclusion key must be a real card role too (slug side warn-checked at
    /// runtime by `warn_unknown_slugs`).
    #[test]
    fn exclusion_roles_are_valid_variants() {
        for (role, _) in ROLE_TAG_EXCLUSIONS {
            assert!(
                CardRole::try_from(*role).is_ok(),
                "'{role}' is not a CardRole variant",
            );
        }
    }

    /// The map covers the otag-derived roles plus the heuristic stragglers and
    /// Tokens as supplements (Protection/Pump/Stax/GraveyardHate/Tokens — for
    /// grouping + union coverage). Only Finisher (dropped) stays out.
    #[test]
    fn map_covers_otag_roles_plus_stragglers() {
        let mapped: Vec<&str> = CATEGORY_ROOTS.iter().map(|(c, _)| *c).collect();
        for supplement in ["pump", "stax", "protection", "graveyard_hate", "tokens"] {
            assert!(
                mapped.contains(&supplement),
                "{supplement} should be otag-mapped as a supplement"
            );
        }
        assert!(
            !mapped.contains(&"finisher"),
            "finisher should not be otag-mapped"
        );
        assert_eq!(mapped.len(), 26);
    }
}
