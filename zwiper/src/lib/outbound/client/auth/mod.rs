//! Authentication API client operations.
//!
//! Provides traits and implementations for user authentication:
//! login, logout, token refresh, and registration.

/// User login endpoint.
pub mod login;
/// User logout endpoint.
pub mod logout;
/// Access token refresh endpoint.
pub mod refresh;
/// New user registration endpoint.
pub mod register;
