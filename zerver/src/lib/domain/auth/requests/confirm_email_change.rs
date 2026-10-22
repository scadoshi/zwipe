//! Switching to a pending email address from the link sent to it.

use zwipe_core::domain::Email;
use zwipe_core::domain::user::User;

/// Request to switch to a pending email address using a one-time token.
pub struct ConfirmEmailChange {
    /// Raw confirmation token (64 hex chars) from the link.
    pub token: String,
}

/// The switch that a confirmation made.
#[derive(Debug)]
pub struct ConfirmedEmailChange {
    /// The user, now on the new address and verified.
    pub user: User,
    /// The address the user had before, which gets told about the change.
    pub old_email: Email,
}

/// Errors that can occur while confirming an email change.
#[derive(Debug, thiserror::Error)]
pub enum ConfirmEmailChangeError {
    /// The token was not found, already used, or has expired.
    #[error("token not found or expired")]
    InvalidToken,
    /// Another account took the address between the request and the click.
    #[error("that address is already in use by another account")]
    AddressTaken,
    /// The stored row did not convert to domain types.
    #[error("email changed but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
    /// Database operation failed.
    #[error(transparent)]
    Database(#[from] anyhow::Error),
}
