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
/// Password reset initiation handler.
pub mod request_password_reset;
/// Re-send email verification handler (private).
pub mod resend_verification;
/// Password reset completion handler.
pub mod reset_password;
/// Session revocation handler.
pub mod revoke_sessions;
/// Email verification handler.
pub mod verify_email;
