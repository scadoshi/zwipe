//! Deck management handlers.

pub mod clear_deck_suppressions;
pub mod clone_deck;
pub mod commander_suggestions;
pub mod create_deck_profile;
pub mod delete_deck;
pub mod get_deck;
pub mod get_deck_profile;
pub mod get_deck_profiles;
pub mod get_deck_tags;
pub mod get_deck_tokens;
pub mod get_shared_deck;
pub mod import_archidekt;
pub mod search_deck_cards;
pub mod share_deck;
pub mod skip_deck_card;
pub mod update_deck_profile;

pub use clear_deck_suppressions::clear_deck_suppressions;
pub use clone_deck::clone_deck;
pub use commander_suggestions::{
    clear_commander_dismissals, dismiss_commander, get_activity, get_commander_suggestions,
    get_play_style_summary, undismiss_commander,
};
pub use create_deck_profile::create_deck_profile;
pub use delete_deck::delete_deck;
pub use get_deck::get_deck;
pub use get_deck_profile::get_deck_profile;
pub use get_deck_profiles::get_deck_profiles;
pub use get_deck_tags::get_deck_tags;
pub use get_deck_tokens::get_deck_tokens;
pub use get_shared_deck::get_shared_deck;
pub use import_archidekt::import_archidekt_deck;
pub use search_deck_cards::search_deck_cards;
pub use share_deck::{share_deck, unshare_deck};
pub use skip_deck_card::{skip_deck_card, unskip_deck_card};
pub use update_deck_profile::update_deck_profile;
