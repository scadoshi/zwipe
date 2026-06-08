//! User metrics HTTP handlers.

/// Helper that re-validates a deck after a mutation and stamps completion.
pub mod check_completion;
/// Returns the caller's lifetime counters.
pub mod get_my_metrics;
/// Returns public app-wide aggregate metrics (no auth).
pub mod get_public_metrics;
/// Accepts a batched usage update from the client.
pub mod record_usage;
