//! Error mappings between SQLx errors and user domain errors.

use thiserror::Error;
use zwipe_core::domain::InvalidEmail;
use zwipe_core::domain::user::username::InvalidUsername;

use crate::domain::user::models::get_user::GetUserError;

/// Errors from converting a database user row into domain types.
#[derive(Debug, Error)]
#[expect(missing_docs, reason = "each variant's message names the failure")]
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
