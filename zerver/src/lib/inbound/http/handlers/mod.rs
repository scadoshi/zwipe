//! HTTP request handlers organized by domain.

/// Authentication and account management handlers.
pub mod auth;
/// Card data handlers.
pub mod card;
/// Public changelog handler (release-history feed).
pub mod changelog;
/// Public client-metadata handlers (app version gating).
pub mod client;
/// Deck management handlers.
pub mod deck;
/// Deck card composition handlers.
pub mod deck_card;
/// Health check handlers.
pub mod health;
/// User metrics handlers.
pub mod metrics;
/// User profile handlers.
pub mod user;
