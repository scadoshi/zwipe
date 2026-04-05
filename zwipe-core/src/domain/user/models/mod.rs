pub mod preferences;
pub mod theme;
pub mod username;

pub use preferences::{
    ALLOWED_THEMES, DARK_ONLY_THEMES, InvalidUpdatePreferences, UpdatePreferences, UserPreferences,
};
pub use username::{InvalidUsername, Username};

use chrono::NaiveDateTime;
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
/// - Authentication responses (as part of Session)
/// - Deck ownership information
///
/// # Example
///
/// ```rust,ignore
/// use zwipe_core::domain::user::User;
///
/// let user = User::new(id, username, email);
/// ```
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct User {
    /// Unique identifier for the user.
    pub id: Uuid,

    /// The user's validated username (3-20 chars, no profanity).
    pub username: Username,

    /// The user's validated email address.
    pub email: EmailAddress,

    /// When the user's email was verified. `None` means not yet verified.
    pub email_verified_at: Option<NaiveDateTime>,
}

impl User {
    /// Creates a new user entity from validated components.
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
            email_verified_at: None,
        }
    }
}
