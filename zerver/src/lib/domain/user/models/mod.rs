//! User domain models and value objects.
//!
//! This module contains the core user entity and related types for user operations.

pub mod get_user;
pub mod username;

use crate::domain::user::models::username::Username;
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A public user profile entity.
///
/// This represents the publicly-visible information about a user. It deliberately
/// excludes sensitive data like password hashes, which are only handled in the
/// authentication layer.
///
/// # Security
///
/// This type is safe to return in API responses and does not contain:
/// - Password or password hash
/// - Session tokens
/// - Private settings
/// - Account creation date (could reveal user enumeration)
///
/// # Usage
///
/// This entity is returned from:
/// - User profile endpoints
/// - Authentication responses (as part of [`Session`](crate::domain::auth::models::session::Session))
/// - Deck ownership information
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::user::models::User;
///
/// // Fetch user
/// let user: User = user_service.get_user(user_id).await?;
///
/// // Safe to send in API response
/// Json(UserResponse {
///     id: user.id,
///     username: user.username,
///     email: user.email,
/// })
/// ```
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct User {
    /// Unique identifier for the user.
    pub id: Uuid,

    /// The user's validated username (3-20 chars, no profanity).
    pub username: Username,

    /// The user's validated email address.
    pub email: EmailAddress,
}

impl User {
    /// Creates a new user entity from validated components.
    ///
    /// This is typically called when reconstructing a user from database records
    /// or converting from internal authentication types.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique user identifier
    /// * `username` - Validated username
    /// * `email` - Validated email address
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let user = User::new(id, username, email);
    /// ```
    pub fn new(id: Uuid, username: Username, email: EmailAddress) -> Self {
        Self {
            id,
            username,
            email,
        }
    }
}
