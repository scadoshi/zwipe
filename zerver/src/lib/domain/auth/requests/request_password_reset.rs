//! Password reset request operation.

/// Request to initiate a password reset for the given email address.
pub struct RequestPasswordReset {
    /// Email address to send the reset link to.
    pub email: String,
}

/// Errors that can occur while initiating a password reset.
///
/// Only database failures are exposed — user-not-found is intentionally
/// swallowed to prevent email enumeration.
#[derive(Debug, thiserror::Error)]
pub enum RequestPasswordResetError {
    /// Database operation failed.
    #[error(transparent)]
    Database(#[from] anyhow::Error),
}
