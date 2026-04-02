//! User profile retrieval operation.
//!
//! Request type re-exported from `zwipe_core`. Service-layer error type remains here.

pub use zwipe_core::domain::user::get_user::*;

#[cfg(feature = "zerver")]
use thiserror::Error;

#[cfg(feature = "zerver")]
/// Errors that can occur when fetching a user.
#[derive(Debug, Error)]
pub enum GetUserError {
    /// No user exists with the requested ID.
    #[error("user not found")]
    NotFound,

    /// Database operation failed while fetching the user.
    #[error(transparent)]
    Database(anyhow::Error),

    /// User was found but database returned invalid/corrupted data.
    #[error("user found but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}
