//! Deck management domain logic.
//!
//! This module provides comprehensive deck building and management for Magic: The Gathering.
//! Users can create decks, add/remove cards, and manage deck configurations like commander
//! selection and copy limits (singleton vs. standard).

/// Deck models and value objects (DeckProfile, Deck, DeckCard, operations).
pub mod models;

/// Port traits (interfaces) for deck operations.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for deck business logic.
#[cfg(feature = "zerver")]
pub mod services;
