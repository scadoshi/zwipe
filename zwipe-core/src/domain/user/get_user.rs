//! User profile retrieval request type.

use std::ops::Deref;
use uuid::Uuid;

/// Request to fetch a user profile by ID.
///
/// # Example
///
/// ```rust,ignore
/// use zwipe_core::domain::user::get_user::GetUser;
/// use uuid::Uuid;
///
/// let request = GetUser::from(user_id);
/// let request = GetUser::new("550e8400-e29b-41d4-a716-446655440000")?;
/// ```
#[derive(Debug, Clone)]
pub struct GetUser {
    /// The unique identifier of the user to fetch.
    pub user_id: Uuid,
}

impl Deref for GetUser {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.user_id
    }
}

impl GetUser {
    /// Creates a new get user request from a UUID string.
    ///
    /// # Errors
    ///
    /// Returns [`uuid::Error`] if the string is not a valid UUID.
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self {
            user_id: Uuid::try_parse(id)?,
        })
    }
}

impl From<Uuid> for GetUser {
    fn from(value: Uuid) -> Self {
        Self { user_id: value }
    }
}
