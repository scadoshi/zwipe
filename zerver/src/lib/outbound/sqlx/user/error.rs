//! Error mappings between `SQLx` errors and user domain errors.

use crate::domain::user::models::get_user::GetUserError;

impl From<sqlx::Error> for GetUserError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}
