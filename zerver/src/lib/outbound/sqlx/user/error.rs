use crate::domain::user::models::{get_user::GetUserError, username::InvalidUsername};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntoUserError {
    #[error(transparent)]
    Username(InvalidUsername),
    #[error(transparent)]
    Email(email_address::Error),
}

impl From<InvalidUsername> for IntoUserError {
    fn from(value: InvalidUsername) -> Self {
        Self::Username(value)
    }
}

impl From<email_address::Error> for IntoUserError {
    fn from(value: email_address::Error) -> Self {
        Self::Email(value)
    }
}

impl From<sqlx::Error> for GetUserError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<IntoUserError> for GetUserError {
    fn from(value: IntoUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}
