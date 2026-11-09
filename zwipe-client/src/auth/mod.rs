//! Authentication API client operations.
//!
//! Methods for user authentication:
//! login, logout, token refresh, and registration.

/// Reset link check endpoint.
pub mod check_reset_token;
/// Pending email change confirmation endpoint.
pub mod confirm_email_change;
/// The browser's login, refresh and logout, carried by an `HttpOnly` cookie.
pub mod cookie;
/// Forgot password / password reset initiation endpoint.
pub mod forgot_password;
/// User login endpoint.
pub mod login;
/// User logout endpoint.
pub mod logout;
/// Access token refresh endpoint.
pub mod refresh;
/// New user registration endpoint.
pub mod register;
/// Resend a pending email change's link endpoint.
pub mod resend_email_change;
/// Resend email verification endpoint.
pub mod resend_verification;
/// Password reset completion endpoint.
pub mod reset_password;
/// Email verification endpoint.
pub mod verify_email;
