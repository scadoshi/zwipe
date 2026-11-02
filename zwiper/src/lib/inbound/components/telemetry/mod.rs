//! Client-side telemetry buffering.
//!
//! Counts swipes and searches in memory, then flushes them to the backend in
//! batches. Avoids one network round-trip per swipe.

/// Pre-auth funnel events, fire-and-forget.
pub mod anonymous;
/// Periodic flusher and screen-exit / drop triggers.
pub mod flush_loop;
/// The shared usage buffer behind one lock.
pub mod usage_buffer;
/// Closed screen/component vocabulary for error-report breadcrumbs.
pub mod vocabulary;
