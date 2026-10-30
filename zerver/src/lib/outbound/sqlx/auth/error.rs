//! Error mappings between SQLx errors and auth domain errors.

use thiserror::Error;
use zwipe_core::domain::InvalidEmail;
use zwipe_core::domain::user::username::InvalidUsername;

use crate::domain::auth::requests::authenticate_user::AuthenticateUserError;
use crate::domain::auth::requests::change_email::ChangeEmailError;
use crate::domain::auth::requests::change_username::ChangeUsernameError;
use crate::domain::auth::requests::confirm_email_change::ConfirmEmailChangeError;
use crate::domain::auth::requests::create_session::CreateSessionError;
use crate::domain::auth::requests::delete_user::DeleteUserError;
use crate::domain::auth::requests::enforce_session_maximum::EnforceSessionMaximumError;
use crate::domain::auth::requests::refresh_session::RefreshSessionError;
use crate::domain::auth::requests::register_user::RegisterUserError;
use crate::domain::auth::requests::revoke_sessions::RevokeSessionsError;
use crate::outbound::sqlx::postgres::IsConstraintViolation;
use crate::outbound::sqlx::user::error::IntoUserError;

/// Errors from converting a database user-with-password-hash row into domain types.
#[derive(Debug, Error)]
#[expect(missing_docs, reason = "each variant's message names the failure")]
pub enum IntoUserWithPasswordHashError {
    #[error(transparent)]
    Username(#[from] InvalidUsername),
    #[error(transparent)]
    Email(#[from] InvalidEmail),
    #[error(transparent)]
    PasswordHash(#[from] argon2::password_hash::Error),
}

impl From<IntoUserError> for RegisterUserError {
    fn from(value: IntoUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<IntoUserWithPasswordHashError> for AuthenticateUserError {
    fn from(value: IntoUserWithPasswordHashError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<IntoUserError> for AuthenticateUserError {
    fn from(value: IntoUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<sqlx::Error> for RegisterUserError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::Database(ref db)
                if db.is_unique_violation() && db.constraint() == Some("users_email_key") =>
            {
                Self::DuplicateEmail
            }
            // The only other unique column is the username.
            e if e.is_unique_constraint_violation() => Self::UsernameTaken,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for AuthenticateUserError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::UserNotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for ChangeUsernameError {
    fn from(value: sqlx::Error) -> Self {
        if value.is_unique_constraint_violation() {
            Self::Duplicate
        } else {
            Self::Database(value.into())
        }
    }
}

impl From<IntoUserError> for ChangeUsernameError {
    fn from(value: IntoUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<sqlx::Error> for ChangeEmailError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<IntoUserError> for ChangeEmailError {
    fn from(value: IntoUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<sqlx::Error> for ConfirmEmailChangeError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<IntoUserError> for ConfirmEmailChangeError {
    fn from(value: IntoUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<sqlx::Error> for DeleteUserError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<sqlx::Error> for RevokeSessionsError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<sqlx::Error> for CreateSessionError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<sqlx::Error> for EnforceSessionMaximumError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<sqlx::Error> for RefreshSessionError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}
