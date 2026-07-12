// Value objects
pub mod board;
pub mod deck_name;
pub mod deck_oracle_tags;
pub mod deck_other_tag;
pub mod deck_tag;
pub mod format;
pub mod import_mode;
pub mod power_level;
pub mod quantity;

// Entities
#[allow(clippy::module_inception)]
pub mod deck;
pub mod deck_card;
pub mod deck_metrics;
pub mod deck_profile;
pub mod deck_warning;

// Domain logic
pub mod validate_deck;

pub use board::{Board, InvalidBoard};
pub use deck::{Deck, DeckEntry};
pub use deck_card::DeckCard;
pub use deck_metrics::DeckMetrics;
pub use deck_name::{DeckName, InvalidDeckname};
pub use deck_oracle_tags::{MAX_DECK_ORACLE_TAGS, dedupe_oracle_tags};
pub use deck_other_tag::{DeckOtherTag, InvalidDeckOtherTag, MAX_DECK_OTHER_TAGS};
pub use deck_profile::DeckProfile;
pub use deck_tag::{DeckTag, InvalidDeckTag, MAX_DECK_TAGS};
pub use deck_warning::{DeckWarning, WarningAction};
pub use format::{Format, InvalidFormat};
pub use import_mode::ImportMode;
pub use power_level::{InvalidPowerLevel, PowerLevel};
pub use quantity::{InvalidQuantity, InvalidUpdateQuanity, Quantity, UpdateQuantity};
pub use validate_deck::{DeckCommandZone, validate_deck};
