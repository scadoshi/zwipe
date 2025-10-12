pub mod access_token;
pub mod authenticate_user;
pub mod change_email;
pub mod change_password;
pub mod change_username;
pub mod delete_user;
pub mod password;
pub mod refresh_token;
pub mod register_user;
pub mod session;

#[cfg(feature = "zerver")]
use crate::domain::auth::models::password::HashedPassword;
#[cfg(feature = "zerver")]
use crate::domain::user::models::{User, Username};
#[cfg(feature = "zerver")]
use email_address::EmailAddress;
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
/// user entity with password hash
/// for authentication operations
#[derive(Debug)]
pub struct UserWithPasswordHash {
    pub id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
    pub password_hash: HashedPassword,
}

#[cfg(feature = "zerver")]
impl From<UserWithPasswordHash> for User {
    fn from(value: UserWithPasswordHash) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
        }
    }
}
