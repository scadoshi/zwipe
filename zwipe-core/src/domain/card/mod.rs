//! Cards: Scryfall data, profiles, roles, oracle tags and search.

pub mod models;
pub use models::{
    Card, card_profile, card_role, keyword, oracle_tag, oracle_tag_examples, scryfall_data,
    search_card, synergy,
};
