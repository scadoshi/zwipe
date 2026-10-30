//! Email verification operation.

/// Request to verify an email address using a one-time token.
pub struct VerifyEmail {
    /// Raw verification token (64 hex chars) sent to the client via email.
    pub token: String,
}

/// Redacted: `token` is a credential.
impl std::fmt::Debug for VerifyEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifyEmail")
            .field("token", &"REDACTED")
            .finish()
    }
}

/// Errors that can occur during email verification.
#[derive(Debug, thiserror::Error)]
pub enum VerifyEmailError {
    /// The token was not found, already used, or has expired.
    #[error("token not found or expired")]
    InvalidToken,
    /// Database operation failed.
    #[error(transparent)]
    Database(#[from] anyhow::Error),
}
