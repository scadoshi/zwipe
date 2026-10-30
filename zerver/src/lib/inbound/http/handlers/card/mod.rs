//! Card data handlers.

pub mod featured_flavor;
pub mod get_artists;
pub mod get_card;
pub mod get_card_roles;
pub mod get_card_types;
pub mod get_keyword_reminders;
pub mod get_keywords;
pub mod get_languages;
pub mod get_oracle_tags;
pub mod get_oracle_words;
pub mod get_printings;
pub mod get_sets;
pub mod get_ub_franchises;
pub mod search_card;
pub mod search_commanders;

pub use featured_flavor::get_featured_flavor;
pub use get_artists::get_artists;
pub use get_card::get_card;
pub use get_card_roles::get_card_roles;
pub use get_card_types::get_card_types;
pub use get_keyword_reminders::get_keyword_reminders;
pub use get_keywords::get_keywords;
pub use get_languages::get_languages;
pub use get_oracle_tags::get_oracle_tags;
pub use get_oracle_words::get_oracle_words;
pub use get_printings::get_printings;
pub use get_sets::get_sets;
pub use get_ub_franchises::get_ub_franchises;
pub use search_card::search_cards;
pub use search_commanders::search_commanders;
