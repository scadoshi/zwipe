//! User profile API client operations.
//!
//! Provides traits and implementations for user account management:
//! fetching user data and updating credentials.

/// Change user email endpoint.
pub mod change_email;
/// Change user password endpoint.
pub mod change_password;
/// Change username endpoint.
pub mod change_username;
/// Delete user account endpoint.
pub mod delete_user;
/// Fetch user profile endpoint.
pub mod get_user;
