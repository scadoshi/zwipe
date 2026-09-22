//! User metrics API client operations.

/// Public marketing counts endpoint.
pub mod public_metrics;
/// Pre-auth funnel event endpoint (unauthenticated).
pub mod record_anonymous_event;
/// Crash report endpoint (unauthenticated).
pub mod record_crash;
/// Batched usage counters endpoint.
pub mod record_usage;
