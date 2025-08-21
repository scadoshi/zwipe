use std::str::FromStr;

use anyhow::Context;
use email_address::EmailAddress;
use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::domain::auth::models::password::HashedPassword;
use crate::domain::auth::models::{
    AuthenticateUserError, ChangePasswordError, ChangePasswordRequest, RegisterUserError,
    RegisterUserRequest, UserWithPasswordHash,
};
use crate::domain::auth::ports::AuthRepository;
use crate::domain::user::models::{User, UserName};
use crate::outbound::sqlx::postgres::Postgres;

// =============================================================================
// DATABASE TYPES
// =============================================================================

/// Raw database user with password hash record - unvalidated data from PostgreSQL
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUserWithPasswordHash {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

/// Converts database user with password hash to validated domain user with password hash
impl TryFrom<DatabaseUserWithPasswordHash> for UserWithPasswordHash {
    type Error = anyhow::Error;

    fn try_from(value: DatabaseUserWithPasswordHash) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id).context("Failed to validate user ID")?;
        let username = UserName::new(&value.username).context("Failed to validate username")?;
        let email = EmailAddress::from_str(&value.email).context("Failed to validate email")?;
        let password_hash = HashedPassword::new(&value.password_hash)
            .context("Failed to validate password hash")?;
        Ok(Self {
            id,
            username,
            email,
            password_hash,
        })
    }
}

// =============================================================================
// REPOSITORY IMPLEMENTATION
// =============================================================================

impl AuthRepository for Postgres {
    async fn create_user_with_password_hash(
        &self,
        req: &RegisterUserRequest,
    ) -> Result<User, RegisterUserError> {
        todo!()
    }

    async fn get_user_with_password_hash(
        &self,
        req: &crate::domain::auth::models::AuthenticateUserRequest,
    ) -> Result<UserWithPasswordHash, AuthenticateUserError> {
        todo!()
    }

    async fn change_password(
        &self,
        req: &ChangePasswordRequest,
    ) -> Result<(), ChangePasswordError> {
        todo!()
    }
}
