//! Card search and filtering.
//!
//! Comprehensive card search with filtering by:
//! - Text: Name, oracle text, type line, flavor text
//! - Mana: CMC, color identity, mana cost
//! - Combat: Power, toughness (exact or range)
//! - Metadata: Rarity, set, artist, language
//! - Flags: Commander-legal, token, digital-only, promo
//! - Pagination: Limit, offset, ordering
//!
//! Uses PostgreSQL full-text search and JSONB operators for efficient querying.

/// Card filter builder for constructing search queries.
pub mod card_filter;
/// Card type enum (Creature, Instant, Sorcery, etc.).
pub mod card_type;
/// Search error types.
#[cfg(feature = "zerver")]
pub mod error;
