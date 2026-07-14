//! Deck management domain logic.
//!
//! This module provides comprehensive deck building and management for Magic: The Gathering.
//! Users can create decks, add/remove cards, and manage deck configurations like commander
//! selection and copy limits (singleton vs. standard).

/// Maximum number of decks a user can own (verified accounts).
pub const MAX_DECKS_PER_USER: i64 = 20;

/// Maximum total card quantity in a single deck, **counting all boards**
/// (mainboard + maybeboard + sideboard), for verified accounts. Counting every
/// board keeps an unbounded maybeboard/sideboard from evading the cap.
pub const MAX_CARDS_PER_DECK: i64 = 500;

/// Maximum decks for accounts with an unverified email address.
pub const UNVERIFIED_MAX_DECKS_PER_USER: i64 = 1;

/// Same per-deck card cap (all boards counted) for accounts with an unverified
/// email address.
pub const UNVERIFIED_MAX_CARDS_PER_DECK: i64 = 100;

/// Deck models and value objects (DeckProfile, Deck, DeckCard, operations).
pub mod models;

/// Port traits (interfaces) for deck operations.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for deck business logic.
#[cfg(feature = "zerver")]
pub mod services;
