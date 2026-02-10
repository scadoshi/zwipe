//! HTTP request handlers organized by domain.

/// Authentication and account management handlers.
pub mod auth;
/// Card data handlers.
pub mod card;
/// Deck management handlers.
pub mod deck;
/// Deck card composition handlers.
pub mod deck_card;
#[cfg(feature = "zerver")]
/// Health check handlers.
pub mod health;
/// User profile handlers.
pub mod user;
