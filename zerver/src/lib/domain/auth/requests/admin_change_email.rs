//! Errors for `zerver-admin change-email`, which switches an account's address at once.

use thiserror::Error;

/// Why an admin email change did not happen.
#[derive(Debug, Error)]
pub enum AdminChangeEmailError {
    /// The account already has that address.
    #[error("the account already has that email")]
    Unchanged,
    /// Another account holds the address.
    #[error("another account has that email")]
    Taken,
    /// The account is gone.
    #[error("the account is gone")]
    NotFound,
    /// A database failure.
    #[error(transparent)]
    Database(#[from] anyhow::Error),
}
