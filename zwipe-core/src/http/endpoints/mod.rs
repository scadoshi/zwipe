//! Every API call, one [`Endpoint`](crate::http::endpoint::Endpoint) impl each.
//!
//! Grouped to mirror [`contracts`]. An endpoint names the
//! method, path, auth and types for one call, so the method a path is used
//! with can't drift from the path itself.

/// Authentication and session endpoints.
pub mod auth;
/// Card catalog and search endpoints.
pub mod card;
/// Deck and deck-card endpoints.
pub mod deck;
/// Service metadata endpoints (changelog, version gate).
pub mod meta;
/// Usage, event and crash reporting endpoints.
pub mod metrics;
/// Suggested commanders and their dismissals.
pub mod suggestions;
/// User profile and preference endpoints.
pub mod user;
