//! Authentication-related components and utilities.
//!
//! Provides session management, automatic token refresh, and logout functionality.

/// Route guard that ensures users are authenticated before accessing protected routes.
pub mod bouncer;
/// Automatic session refresh and token upkeep in the background.
pub mod session_upkeep;
/// Reactive logout functionality for session signals.
pub mod signal_logout;
