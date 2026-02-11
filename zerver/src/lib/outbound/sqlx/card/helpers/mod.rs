//! SQL generation helpers for card data operations.

/// Field listing and binding for the 94-column `scryfall_data` table.
pub mod scryfall_data_fields;
/// Upsert strategies: single, bulk, batch, and delta-aware.
pub mod upsert_card;
