// Value objects
pub mod deck_name;
pub mod format;
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

pub use deck::{Deck, DeckEntry};
pub use deck_card::DeckCard;
pub use deck_metrics::DeckMetrics;
pub use deck_name::{DeckName, InvalidDeckname};
pub use deck_profile::DeckProfile;
pub use deck_warning::{DeckWarning, WarningAction};
pub use format::{Format, InvalidFormat};
pub use quantity::{InvalidQuantity, InvalidUpdateQuanity, Quantity, UpdateQuantity};
pub use validate_deck::{validate_deck, DeckCommandZone};
