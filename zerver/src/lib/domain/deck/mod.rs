//! Deck management domain logic.
//!
//! This module provides deck building and management for Magic: The Gathering.
//! Users can create decks, add/remove cards, and manage deck configurations like commander
//! selection and copy limits (singleton vs. standard).

/// Maximum number of decks a user can own (verified accounts).
pub const MAX_DECKS_PER_USER: i64 = 20;

pub use zwipe_core::domain::deck::{
    MAX_CARDS_PER_DECK, MAX_COMMANDER_MAYBEBOARD, UNVERIFIED_MAX_CARDS_PER_DECK,
    UNVERIFIED_MAX_DECKS_PER_USER,
};

/// Deck models and value objects (`DeckProfile`, `Deck`, `DeckCard`, operations).
pub mod models;

/// Port traits (interfaces) for deck operations.
pub mod ports;

/// Service implementations for deck business logic.
pub mod services;
