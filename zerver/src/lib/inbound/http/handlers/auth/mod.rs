//! Authentication and account management handlers.

/// Login handler.
pub mod authenticate_user;
/// Email change handler.
pub mod change_email;
/// Password change handler.
pub mod change_password;
/// Username change handler.
pub mod change_username;
/// Account deletion handler.
pub mod delete_user;
/// Session refresh handler.
pub mod refresh_session;
/// Registration handler.
pub mod register_user;
/// Session revocation handler.
pub mod revoke_sessions;
