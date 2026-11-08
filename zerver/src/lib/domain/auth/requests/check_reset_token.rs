//! Checking a password reset token without spending it.

/// Errors that can occur while checking a password reset token.
#[derive(Debug, thiserror::Error)]
pub enum CheckResetTokenError {
    /// The token was not found, already used, or has expired.
    #[error("token not found or expired")]
    InvalidToken,
    /// Database operation failed.
    #[error(transparent)]
    Database(#[from] anyhow::Error),
}
