//! User metrics domain — counters, events, and audit log.
//!
//! Vanity dashboard totals and high-volume usage signals. The client buffers
//! swipe / search counts and flushes them periodically; rare events
//! (signup, deck created/completed) and credential audit entries are
//! written inline at the relevant call sites.

/// Metric models and error types.
pub mod models;

/// Port traits for metrics persistence.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for metrics business logic.
#[cfg(feature = "zerver")]
pub mod services;
