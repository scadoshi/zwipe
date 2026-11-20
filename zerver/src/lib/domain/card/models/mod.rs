/// Helper traits and utilities for card operations.
pub mod helpers;
/// Removing the printings Scryfall deleted: the pure plan zervice applies.
pub mod printing_removal;
/// The checks a Scryfall pull passes before zervice writes any of it.
pub mod scryfall_pull;
/// Card search error types.
pub mod search_card;

/// Commander synergy payload (cache read side).
pub mod synergy;

/// Zervice metrics tracking for sync and classification operations.
pub mod zervice_metrics;

/// One recorded zervice step: when, how long, how it ended and what it counted.
pub mod zervice_step;
