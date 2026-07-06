//! Metrics domain models and error types.

/// Service-layer error types.
#[cfg(feature = "zerver")]
pub mod errors;
/// Sparse event kinds and audit actions written one row at a time.
pub mod kinds;
/// Per-user lifetime counter aggregate.
pub mod lifetime_counters;
/// App-wide aggregate metrics shown on the marketing site.
pub mod public_metrics;
