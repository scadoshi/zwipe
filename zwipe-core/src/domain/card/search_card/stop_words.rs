//! Stop words for card text tokenization.
//!
//! Single source of truth shared between the frontend (`deck_cards.rs`) and
//! the backend SQL adapter (`get_card_types`, `get_oracle_words`).
//!
//! Note: `query_scalar!` requires literal SQL strings so the SQL queries
//! reference these lists in comments rather than importing them directly.
//! Keep the SQL `NOT IN` clauses in sync with these constants.

/// Stop words filtered out of `type_line` tokens.
pub const TYPE_STOP_WORDS: &[&str] = &["//", "-", "the", "of", "and/or", "you", "you'll"];

/// Stop words filtered out of oracle text tokens.
pub const ORACLE_STOP_WORDS: &[&str] = &[
    "a", "an", "the", "of", "to", "in", "on", "at", "by", "for", "with", "from", "into", "as",
    "and", "or", "but", "that", "which", "who", "it", "its", "you", "your", "this", "those",
    "these", "they", "their", "is", "are", "was", "be", "has", "have", "do", "does", "been", "",
];
