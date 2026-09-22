# Phase 3: serve which oracle tags are curated

**Status: DONE 2026-09-22, rides 1.10.2.**

`CURATED_ORACLE_TAGS` is a hand-picked shortlist of about 24 slugs the otag pickers show before the user types anything: the original mechanical categories mapped to their best-populated real slug, plus functional fills. Its own doc says "tuned over time", and every tune currently costs a store train.

Read in exactly one place, `screens/deck/components/oracle_tag_select.rs`, which is shared by deck create and deck edit.

## Do it as a field, not an endpoint

The obvious move is a fourth endpoint like phase 1. Do not. The picker already fetches the full oracle-tag catalog, and curation is a property of a tag, not a separate list. Add it to what is already on the wire:

```rust
pub struct OracleTag {
    pub slug: String,
    pub label: String,
    pub description: Option<String>,
    pub parent_slugs: Vec<String>,
    /// Surfaced in pickers before the user searches. Tuned server-side.
    #[serde(default)]
    pub curated: bool,
}
```

zerver fills it from the compiled `CURATED_ORACLE_TAGS` when building the catalog response. The picker's empty-query branch filters on `t.curated` instead of intersecting against a compiled slug list, which is less code than it has now.

This also fixes something the current version has to work around. Today the picker filters the curated slugs down to "entries the backend still serves", because a curated slug can be retired server-side while the compiled list still names it. With the flag on the tag, an unserved tag cannot be curated by construction.

## Why the flag comes from the const, not a column

The tempting version is a `curated` column on `oracle_tags`, so tuning is a SQL `UPDATE` with no deploy at all. It does not work as written: zervice rebuilds that table nightly with `DELETE FROM oracle_tags` followed by a bulk insert (`outbound/sqlx/card/helpers/oracle_tags.rs`), so the column is wiped every night.

Making it work needs a side table keyed by slug, joined at serve time and deliberately not touched by the sync. That is a migration, a join and a new thing to remember, to save a deploy that takes minutes. Not worth it.

Keep the const. It satisfies the rule: zerver reads it, zwiper does not.

## Wire safety

Adding a field is safe in both directions here.

`OracleTag` has no `deny_unknown_fields`, so shipped clients ignore `curated` and keep using their compiled list. Nothing breaks for anyone on 1.10.1 or earlier.

`#[serde(default)]` covers the other direction: a client built after this change still decodes a response from a server that predates it, which matters if a deploy is ever rolled back. Without the default, a missing bool is a decode error and the whole catalog fails.

## Steps

1. Add the field to `OracleTag` with `#[serde(default)]`.
2. zerver sets it when assembling the catalog, from `CURATED_ORACLE_TAGS`.
3. `oracle_tag_select.rs` filters on the flag; delete the import, the intersect, and the now-redundant still-served filter.
4. Leave `CURATED_ORACLE_TAGS` in core. zerver reads it; zwiper no longer can.

## Verification

- A tag marked curated server-side appears in the picker's default grid without a client build. Easiest check: flip one entry, deploy, reopen the picker.
- Selected-but-not-curated slugs still pin to the grid, which is existing behaviour worth not regressing.
- An old client keeps its compiled grid and shows no change at all.
