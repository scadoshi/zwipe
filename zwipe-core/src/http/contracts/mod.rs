//! HTTP request/response contract structs shared between frontend and backend.

/// Authentication request contracts.
pub mod auth;
/// Changelog contracts (release-history feed).
pub mod changelog;
/// Client metadata contracts (app version gating).
pub mod client;
/// Deck management request contracts.
pub mod deck;
/// Deck card operation request contracts.
pub mod deck_card;
/// User metrics request and response contracts.
pub mod metrics;
/// User account request contracts.
pub mod user;
