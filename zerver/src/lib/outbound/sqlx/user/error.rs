//! Error mappings between SQLx errors and user domain errors.

use crate::domain::user::models::get_user::GetUserError;
use zwipe_core::domain::{user::username::InvalidUsername, InvalidEmail};
use thiserror::Error;

/// Errors from converting a database user row into domain types.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum IntoUserError {
    #[error(transparent)]
    Username(#[from] InvalidUsername),
    #[error(transparent)]
    Email(#[from] InvalidEmail),
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
