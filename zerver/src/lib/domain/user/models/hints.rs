//! One-time UI hint marking operation.
//!
//! Service-layer error type. The validated request type ([`MarkHintShown`])
//! and key constants live in zwipe-core (`domain::user::models::hints`).

#[cfg(feature = "zerver")]
use thiserror::Error;

#[cfg(feature = "zerver")]
#[allow(unused_imports)]
use zwipe_core::domain::user::models::hints::MarkHintShown;

/// Errors that can occur when marking a hint as shown.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum MarkHintShownError {
    /// No user exists with the requested ID.
    #[error("user not found")]
    NotFound,

    /// Database operation failed while updating the hint map.
    #[error(transparent)]
    Database(anyhow::Error),

    /// User was updated but database returned invalid/corrupted data.
    #[error("user found but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}
