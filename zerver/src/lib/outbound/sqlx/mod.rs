//! SQLx repository implementations for PostgreSQL.

/// Authentication and account management repositories.
pub mod auth;
/// Card data and Scryfall sync repositories.
pub mod card;
/// Deck management repositories.
pub mod deck;
/// Health check repository.
pub mod health;
/// Shared PostgreSQL connection pool and constraint helpers.
pub mod postgres;
/// User profile repository.
pub mod user;
