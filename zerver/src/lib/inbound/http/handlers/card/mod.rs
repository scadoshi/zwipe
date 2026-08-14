//! Card data handlers.

/// Featured flavor card handler (one shared pick per UTC hour).
pub mod featured_flavor;
/// Distinct artist names handler.
pub mod get_artists;
/// Single card lookup handler.
pub mod get_card;
/// Card-role catalog handler (`GET /api/card/roles`).
pub mod get_card_roles;
/// Distinct card type names handler.
pub mod get_card_types;
/// Keyword-reminder catalog handler (name → reminder text).
pub mod get_keyword_reminders;
/// Distinct keyword ability names handler.
pub mod get_keywords;
/// Distinct language names handler.
pub mod get_languages;
/// Oracle tag catalog handler.
pub mod get_oracle_tags;
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
