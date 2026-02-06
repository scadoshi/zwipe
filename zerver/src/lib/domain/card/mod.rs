//! Magic: The Gathering card data and search.
//!
//! This module manages the comprehensive card database sourced from Scryfall API,
//! providing search, retrieval, and synchronization capabilities for MTG cards.
//!
//! # Card Data Source
//!
//! All card data comes from the [Scryfall API](https://scryfall.com/docs/api).

/// Card models and value objects (ScryfallData, CardProfile, Card, filters).
pub mod models;

/// Port traits (interfaces) for card operations.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for card business logic.
#[cfg(feature = "zerver")]
pub mod services;
