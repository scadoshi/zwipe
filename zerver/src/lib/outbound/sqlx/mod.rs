//! SQLx repository implementations for PostgreSQL.

/// Authentication and account management repositories.
pub mod auth;
/// Card data and Scryfall sync repositories.
pub mod card;
/// Deck management repositories.
pub mod deck;
/// Health check repository.
pub mod health;
/// User metrics repository.
pub mod metrics;
/// Shared PostgreSQL connection pool and constraint helpers.
pub mod postgres;
/// Nightly upkeep (maintenance) deletes — zervice-only.
pub mod upkeep;
/// User profile repository.
pub mod user;
