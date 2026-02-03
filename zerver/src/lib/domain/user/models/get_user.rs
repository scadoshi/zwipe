//! User profile retrieval operation.
//!
//! This module provides the request type and errors for fetching user profile
//! information by user ID.
//!
//! # Use Cases
//!
//! - Fetching user profile for display
//! - Resolving user IDs to usernames/emails
//! - Verifying user existence before operations
//! - Loading user data for authentication
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::user::models::get_user::GetUser;
//!
//! // Fetch user by ID
//! let request = GetUser::from(user_id);
//! let user = user_service.get_user(request).await?;
//!
//! println!("{} ({})", user.username, user.email);
//! ```

#[cfg(feature = "zerver")]
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
/// Errors that can occur when fetching a user.
///
/// User retrieval is a simple operation that either succeeds (user found) or
/// fails (user not found or database error).
#[derive(Debug, Error)]
pub enum GetUserError {
    /// No user exists with the requested ID.
    ///
    /// This is the expected failure case when a user ID doesn't exist or
    /// has been deleted.
    #[error("user not found")]
    NotFound,

    /// Database operation failed while fetching the user.
    #[error(transparent)]
    Database(anyhow::Error),

    /// User was found but database returned invalid/corrupted data.
    ///
    /// This indicates database schema issues or data corruption.
    #[error("user found but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}

/// Request to fetch a user profile by ID.
///
/// This is a simple request type that wraps a user ID. The service layer
/// fetches the corresponding user from the repository.
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::user::models::get_user::GetUser;
/// use uuid::Uuid;
///
/// // From UUID directly
/// let request = GetUser::from(user_id);
///
/// // From string (parses UUID)
/// let request = GetUser::new("550e8400-e29b-41d4-a716-446655440000")?;
///
/// // Fetch user
/// let user = user_service.get_user(request).await?;
/// ```
#[derive(Debug, Clone)]
pub struct GetUser {
    /// The unique identifier of the user to fetch.
    pub user_id: Uuid,
}

impl GetUser {
    /// Creates a new get user request from a UUID string.
    ///
    /// # Arguments
    ///
    /// * `id` - UUID string of the user to fetch
    ///
    /// # Errors
    ///
    /// Returns [`uuid::Error`] if the string is not a valid UUID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let request = GetUser::new("550e8400-e29b-41d4-a716-446655440000")?;
    /// ```
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self {
            user_id: Uuid::try_parse(id)?,
        })
    }
}

impl From<Uuid> for GetUser {
    /// Creates a get user request from a UUID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let request = GetUser::from(user_id);
    /// ```
    fn from(value: Uuid) -> Self {
        Self { user_id: value }
    }
}
