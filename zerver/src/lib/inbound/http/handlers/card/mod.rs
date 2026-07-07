//! Card data handlers.

/// Distinct artist names handler.
pub mod get_artists;
/// Single card lookup handler.
pub mod get_card;
/// Distinct card type names handler.
pub mod get_card_types;
/// Distinct keyword ability names handler.
pub mod get_keywords;
/// Distinct language names handler.
pub mod get_languages;
/// Distinct oracle text word names handler.
pub mod get_oracle_words;
/// All printings of a card by oracle ID.
pub mod get_printings;
/// Distinct set names handler.
pub mod get_sets;
/// Card search handler.
pub mod search_card;
/// Commander search handler (popularity-ordered, banded, wildcarded).
pub mod search_commanders;
