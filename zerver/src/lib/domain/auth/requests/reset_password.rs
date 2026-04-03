//! Password reset completion operation.

/// Errors that can occur while completing a password reset.
#[derive(Debug, thiserror::Error)]
pub enum ResetPasswordError {
    /// The token was not found, already used, or has expired.
    #[error("token not found or expired")]
    InvalidToken,
    /// The new password failed validation.
    #[error("invalid password: {0}")]
    InvalidPassword(String),
    /// Database operation failed.
    #[error(transparent)]
    Database(#[from] anyhow::Error),
}

#[cfg(feature = "zerver")]
/// Request to complete a password reset using a one-time token.
pub struct ResetPassword {
    /// Raw reset token (64 hex chars) provided by the client.
    pub token: String,
    /// Validated and hashed new password, ready for database storage.
    pub new_password_hash: crate::domain::auth::models::password::HashedPassword,
}

#[cfg(feature = "zerver")]
impl ResetPassword {
    /// Validates the new password and hashes it.
    ///
    /// # Errors
    ///
    /// Returns [`ResetPasswordError::InvalidPassword`] if the password fails policy,
    /// or [`ResetPasswordError::Database`] if hashing fails.
    pub fn new(
        token: String,
        new_password: impl AsRef<str>,
    ) -> Result<Self, ResetPasswordError> {
        use crate::domain::auth::models::password::{HashedPassword, Password};

        let password = Password::new(new_password)
            .map_err(|e| ResetPasswordError::InvalidPassword(e.to_string()))?;
        let new_password_hash = HashedPassword::generate(password)
            .map_err(|e| ResetPasswordError::Database(e.into()))?;
        Ok(Self {
            token,
            new_password_hash,
        })
    }
}
