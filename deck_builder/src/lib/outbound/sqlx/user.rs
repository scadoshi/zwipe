// =============================================================================
// IMPORTS
// =============================================================================

use anyhow::{anyhow, Context};
use email_address::EmailAddress;
use sqlx_macros::FromRow;

use crate::domain::auth::models::password::HashedPassword;
use crate::domain::user::models::{
    User, UserAuthenticationError, UserAuthenticationRequest, UserCreationError,
    UserCreationRequest, UserId, UserName,
};
use crate::domain::user::ports::UserRepository;
use crate::outbound::sqlx::postgres::Postgres;

// =============================================================================
// DATABASE TYPES
// =============================================================================

/// Raw database user record - unvalidated data from PostgreSQL
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUser {
    pub id: i32,
    pub username: String,
    pub email: String,
}

/// Converts database user to validated domain user
impl TryFrom<DatabaseUser> for User {
    type Error = anyhow::Error;

    fn try_from(value: DatabaseUser) -> Result<Self, Self::Error> {
        let id = UserId::new(value.id).context("Failed to validate user ID")?;
        let username = UserName::new(&value.username).context("Failed to validate username")?;
        let email =
            EmailAddress::parse_with_options(&value.email, email_address::Options::default())
                .context("Failed to validate email")?;
        Ok(Self {
            id,
            username,
            email,
        })
    }
}

// =============================================================================
// REPOSITORY IMPLEMENTATION
// =============================================================================

impl UserRepository for Postgres {
    /// Creates user with transaction safety and domain validation
    async fn create_user(&self, req: &UserCreationRequest) -> Result<User, UserCreationError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| UserCreationError::DatabaseError(anyhow!("{}", e)))?;

        let database_user = self.save_user(&mut tx, req).await?;

        let user = User::try_from(database_user)
            .map_err(|e| UserCreationError::InvalidUserFromDatabase(anyhow!("{e}")))?;

        tx.commit().await.map_err(|e| {
            UserCreationError::DatabaseError(anyhow!(
                "Failed to commit PostgreSQL transaction: {e}"
            ))
        });

        Ok(user)
    }

    /// Gets password hash for authentication by username or email
    async fn get_user_password_hash(
        &self,
        req: &UserAuthenticationRequest,
    ) -> Result<HashedPassword, UserAuthenticationError> {
        let raw_password_hash = self
            .get_user_password_hash_with_username_or_email(&self.pool, &req.identifier)
            .await?;

        HashedPassword::new(&raw_password_hash)
            .map_err(|e| UserAuthenticationError::InvalidHashFromDatabase(anyhow!("{e}")))
    }
}
