//! Authentication-related components and utilities.
//!
//! Provides session management, automatic token refresh, and logout functionality.

/// Route guard that ensures users are authenticated before accessing protected routes.
pub mod bouncer;
/// Awaitable session freshness guard with single-flight refresh.
pub mod ensure_session;
/// Background session upkeep loop and app context providers.
pub mod session_upkeep;
/// Reactive logout functionality for session signals.
pub mod signal_logout;
