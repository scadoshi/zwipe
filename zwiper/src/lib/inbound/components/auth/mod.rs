//! Authentication-related components and utilities.
//!
//! Provides session management, automatic token refresh, and logout functionality.

/// Router layout that gates all authed routes behind a valid session.
pub mod auth_gate;
/// Awaitable session freshness guard with single-flight refresh.
pub mod ensure_session;
/// Background session upkeep loop and app context providers.
pub mod session_upkeep;
/// Reactive logout functionality for session signals.
pub mod signal_logout;
