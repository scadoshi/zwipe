//! SQL generation helpers for card data operations.

/// Derive `mechanical_categories` from oracle-tag subtrees + `all_parts` (Phase 2).
pub mod derive_categories;
/// Our authored oracle-tag descriptions, overlaid onto the catalog each sync.
pub mod oracle_tag_descriptions;
/// Group each card's oracle tags under their role + an "other" bucket (Phase 2).
pub mod oracle_tag_groups;
/// Oracle Tags ingest: replace the oracle_tag catalog + card correlation.
pub mod oracle_tags;
/// Field listing and binding for the 94-column `scryfall_data` table.
pub mod scryfall_data_fields;
/// Upsert strategies: single, bulk, batch, and delta-aware.
pub mod upsert_card;
