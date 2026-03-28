//! Deck management domain logic.
//!
//! This module provides comprehensive deck building and management for Magic: The Gathering.
//! Users can create decks, add/remove cards, and manage deck configurations like commander
//! selection and copy limits (singleton vs. standard).

/// Maximum number of decks a user can own (free tier cap).
pub const MAX_DECKS_PER_USER: i64 = 20;

/// Maximum total card quantity across all cards in a single deck (free tier cap).
pub const MAX_CARDS_PER_DECK: i64 = 250;

/// Deck models and value objects (DeckProfile, Deck, DeckCard, operations).
pub mod models;

/// Port traits (interfaces) for deck operations.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for deck business logic.
#[cfg(feature = "zerver")]
pub mod services;
