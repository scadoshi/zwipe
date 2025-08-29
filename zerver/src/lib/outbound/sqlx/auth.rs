use std::str::FromStr;

use anyhow::{anyhow, Context};
use email_address::EmailAddress;
use sqlx::{query, query_as};
use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::domain::auth::models::password::HashedPassword;
use crate::domain::auth::models::{
    AuthenticateUserError, AuthenticateUserRequest, ChangePasswordError, ChangePasswordRequest, RegisterUserError, RegisterUserRequest, UserWithPasswordHash
};
use crate::domain::auth::ports::AuthRepository;
use crate::domain::user::models::{User, UserName};
use crate::outbound::sqlx::postgres::{IsUniqueConstraintViolation, Postgres};
use crate::outbound::sqlx::user::DatabaseUser;

// ==========
//  db types
// ==========

/// raw database user with password hash record 
/// (unvalidated data from `PostgreSQL`)
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUserWithPasswordHash {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
}

/// converts database user with password hash
/// to validated domain user with password hash
impl TryFrom<DatabaseUserWithPasswordHash> for UserWithPasswordHash {
    type Error = anyhow::Error;

    fn try_from(value: DatabaseUserWithPasswordHash) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id).context("failed to validate user id")?;
        let username = UserName::new(&value.username).context("failed to validate username")?;
        let email = EmailAddress::from_str(&value.email).context("failed to validate email")?;

        let password_hash: Option<HashedPassword> = value
            .password_hash
            .map(|password_hash| HashedPassword::new(&password_hash).context("failed to create validate password hash"))
            .transpose()?;

        Ok(Self {
            id,
            username,
            email,
            password_hash,
        })
    }
}

impl AuthRepository for Postgres {
    // ========
    //  create
    // ========
    async fn create_user_with_password_hash(
        &self,
        request: &RegisterUserRequest,
    ) -> Result<User, RegisterUserError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RegisterUserError::DatabaseIssues(anyhow!("{e}")))?;

        let database_user = query_as!(
            DatabaseUser, 
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email", 
            request.username.to_string(), 
            request.email.to_string(), 
            request.password_hash.to_string()
        ).fetch_one(&mut *tx).await.map_err(|e| {
            if e.is_unique_constraint_violation() {
                return RegisterUserError::Duplicate;
            }
            RegisterUserError::DatabaseIssues(anyhow!("{e}"))
        })?;

        let user: User = database_user
            .try_into()
            .map_err(|e| RegisterUserError::InvalidUserFromDatabase(anyhow!("{e}")))?;

        tx.commit()
            .await
            .map_err(|e| RegisterUserError::DatabaseIssues(anyhow!("{e}")))?;

        Ok(user)
    }
    // =====
    //  get
    // =====
    async fn get_user_with_password_hash(
        &self,
        request: &AuthenticateUserRequest,
    ) -> Result<UserWithPasswordHash, AuthenticateUserError> {

        let database_user: DatabaseUserWithPasswordHash = query_as!(
            DatabaseUserWithPasswordHash,
            "SELECT id, username, email, password_hash FROM users WHERE (username = $1 OR email = $1)",
            request.identifier
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AuthenticateUserError::UserNotFound, 
            e => AuthenticateUserError::DatabaseIssues(anyhow!("{e}")),
        })?;

        let user: UserWithPasswordHash = database_user
            .try_into()
            .map_err(|e| AuthenticateUserError::InvalidUserFromDatabase(anyhow!("{e}")))?;

        Ok(user)
    }
    // ========
    //  update                           
    // ========
    async fn change_password(
        &self,
        request: &ChangePasswordRequest,
    ) -> Result<(), ChangePasswordError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ChangePasswordError::DatabaseIssues(anyhow!("{e}")))?;

        query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2", 
            request.password_hash.to_string(), 
            request.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ChangePasswordError::UserNotFound,
            e => ChangePasswordError::DatabaseIssues(anyhow!("{e}")),
        })?;

        tx.commit()
            .await
            .map_err(|e| ChangePasswordError::DatabaseIssues(anyhow!("{e}")))?;

        Ok(())
    }
}
