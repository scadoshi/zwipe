use crate::domain::auth::models::password::InvalidPassword;
use thiserror::Error;

#[cfg(feature = "zerver")]
use crate::domain::auth::models::{
    authenticate_user::AuthenticateUserError, password::HashedPassword,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;

/// errors encountered while constructing `ChangePasswordRequestError`
#[derive(Debug, Error)]
pub enum InvalidChangePassword {
    #[error(transparent)]
    Password(InvalidPassword),
    #[error("failed to hash password: {0}")]
    FailedPasswordHash(anyhow::Error),
}

#[cfg(feature = "zerver")]
/// errors encountered while changing password
#[derive(Debug, Error)]
pub enum ChangePasswordError {
    #[error("user not found")]
    UserNotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error(transparent)]
    AuthenticateUserError(AuthenticateUserError),
}

#[cfg(feature = "zerver")]
impl From<AuthenticateUserError> for ChangePasswordError {
    fn from(value: AuthenticateUserError) -> Self {
        Self::AuthenticateUserError(value)
    }
}

#[cfg(feature = "zerver")]
impl From<sqlx::Error> for ChangePasswordError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::UserNotFound,
            e => Self::Database(e.into()),
        }
    }
}

#[cfg(feature = "zerver")]
/// change password request
/// with idenifier and new password hash
#[derive(Debug)]
pub struct ChangePassword {
    pub user_id: Uuid,
    pub current_password: String,
    pub new_password_hash: HashedPassword,
}
#[cfg(feature = "zerver")]
impl ChangePassword {
    pub fn new(
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
    ) -> Result<Self, InvalidChangePassword> {
        use crate::domain::auth::models::password::Password;

        let new_password = Password::new(new_password).map_err(InvalidChangePassword::Password)?;
        // no type validation of current password
        // so user isn't locked out of changing their password
        let current_password = current_password.to_string();
        let new_password_hash = HashedPassword::generate(new_password)
            .map_err(|e| InvalidChangePassword::FailedPasswordHash(e.into()))?;

        Ok(Self {
            user_id,
            current_password,
            new_password_hash,
        })
    }
}
