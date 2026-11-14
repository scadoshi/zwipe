//! Error mappings between `SQLx` errors and user domain errors.

use crate::domain::user::models::get_user::GetUserError;

impl From<sqlx::Error> for GetUserError {
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "sqlx::Error is a foreign enum: every variant but the ones handled is a database failure"
    )]
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}
