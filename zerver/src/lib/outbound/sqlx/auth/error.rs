use crate::{
    domain::{
        auth::models::{
            authenticate_user::AuthenticateUserError,
            change_email::ChangeEmailError,
            change_username::ChangeUsernameError,
            delete_user::DeleteUserError,
            register_user::RegisterUserError,
            session::{
                create_session::CreateSessionError,
                delete_expired_sessions::DeleteExpiredSessionsError,
                enforce_session_maximum::EnforceSessionMaximumError,
                refresh_session::RefreshSessionError, revoke_sessions::RevokeSessionsError,
            },
        },
        user::models::username::InvalidUsername,
    },
    outbound::sqlx::{postgres::IsConstraintViolation, user::error::IntoUserError},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntoUserWithPasswordHashError {
    #[error(transparent)]
    Username(InvalidUsername),
    #[error(transparent)]
    Email(email_address::Error),
    #[error(transparent)]
    PasswordHash(argon2::password_hash::Error),
}

impl From<argon2::password_hash::Error> for IntoUserWithPasswordHashError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::PasswordHash(value)
    }
}

impl From<InvalidUsername> for IntoUserWithPasswordHashError {
    fn from(value: InvalidUsername) -> Self {
        Self::Username(value)
    }
}

impl From<email_address::Error> for IntoUserWithPasswordHashError {
    fn from(value: email_address::Error) -> Self {
        Self::Email(value)
    }
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
            e if e.is_unique_constraint_violation() => Self::Duplicate,
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
        if value.is_unique_constraint_violation() {
            Self::Duplicate
        } else {
            Self::Database(value.into())
        }
    }
}

impl From<IntoUserError> for ChangeEmailError {
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

impl From<sqlx::Error> for DeleteExpiredSessionsError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<sqlx::Error> for RefreshSessionError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}
