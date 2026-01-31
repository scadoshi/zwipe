use crate::domain::user::models::{get_user::GetUserError, username::InvalidUsername};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntoUserError {
    #[error(transparent)]
    Username(#[from] InvalidUsername),
    #[error(transparent)]
    Email(#[from] email_address::Error),
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
