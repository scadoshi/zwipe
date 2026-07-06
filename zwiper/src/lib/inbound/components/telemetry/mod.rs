//! Client-side telemetry buffering.
//!
//! Counts swipes and searches in memory, then flushes them to the backend in
//! batches. Avoids one network round-trip per swipe.

/// Pre-auth funnel events, fire-and-forget.
pub mod anonymous;
/// Periodic flusher and screen-exit / drop triggers.
pub mod flush_loop;
/// Atomic in-memory counters.
pub mod usage_buffer;
