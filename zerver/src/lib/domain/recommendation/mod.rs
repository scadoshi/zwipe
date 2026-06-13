//! Recommendation domain: the [`CardRecommender`](ports::CardRecommender)
//! outbound port and its request/result models.
//!
//! Feeds the live, deck-aware synergy ordering for mature decks (the cached
//! commander signal handles smaller decks and is the universal fallback). The
//! port is vendor-neutral; the `Recommander` adapter in `outbound/recommander`
//! implements it. See `plans/recommander-integration.md`.

/// Recommendation request + result models.
pub mod models;
/// [`CardRecommender`](ports::CardRecommender) outbound port.
pub mod ports;
