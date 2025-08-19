use anyhow::{anyhow, Context};
use email_address::EmailAddress;
use sqlx::{query, query_as, QueryBuilder};
use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::domain::user::models::{
    CreateUserError, CreateUserRequest, DeleteUserError, DeleteUserRequest, GetUserError,
    GetUserRequest, UpdateUserError, UpdateUserRequest, User, UserName,
};
use crate::domain::user::ports::UserRepository;
use crate::outbound::sqlx::postgresql::{IsUniqueConstraintViolation, Postgres};

// =============================================================================
// DATABASE TYPES
// =============================================================================

/// Raw database user record - unvalidated data from PostgreSQL
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUser {
    pub id: String,
    pub username: String,
    pub email: String,
}

/// Converts database user to validated domain user
impl TryFrom<DatabaseUser> for User {
    type Error = anyhow::Error;

    fn try_from(value: DatabaseUser) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id).context("Failed to validate user ID")?;
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
//                          REPOSITORY IMPLEMENTATION
// =============================================================================

impl UserRepository for Postgres {
    //
    // =============================================================================
    //                                   CREATE
    // =============================================================================
    //
    async fn create_user(&self, req: &CreateUserRequest) -> Result<User, CreateUserError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CreateUserError::DatabaseIssues(anyhow!("{e}")))?;

        let database_user = query_as!(
            DatabaseUser,
            "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id, username, email",
            req.username.to_string(),
            req.email.to_string(),
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if e.is_unique_constraint_violation() {
                return CreateUserError::Duplicate;
            }
            CreateUserError::DatabaseIssues(anyhow!("{e}"))
        })?;

        let user: User = database_user
            .try_into()
            .map_err(|e| CreateUserError::InvalidUserFromDatabase(anyhow!("{e}")))?;

        tx.commit()
            .await
            .map_err(|e| CreateUserError::DatabaseIssues(anyhow!("{e}")))?;

        Ok(user)
    }
    //
    // =============================================================================
    //                                     GET
    // =============================================================================
    //
    async fn get_user(&self, req: &GetUserRequest) -> Result<User, GetUserError> {
        // tries to find user by id first
        let database_user = query_as!(
            DatabaseUser,
            "SELECT id, username, email FROM users WHERE (id::text = $1 OR username = $1 OR email = $1)",
            req.identifier
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => GetUserError::NotFound,
            e => GetUserError::DatabaseIssues(anyhow!("{e}")),
        })?;

        let user: User = database_user
            .try_into()
            .map_err(|e| GetUserError::InvalidUserFromDatabase(anyhow!("{e}")))?;

        Ok(user)
    }
    //
    // =============================================================================
    //                                     UPDATE
    // =============================================================================
    //
    async fn update_user(&self, req: &UpdateUserRequest) -> Result<User, UpdateUserError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| UpdateUserError::DatabaseIssues(anyhow!("{e}")))?;

        let mut query_builder = QueryBuilder::new("UPDATE users SET ");

        if let Some(username) = &req.username {
            query_builder.push(format!("username = {}", username));
        }
        if let Some(email) = &req.email {
            query_builder.push(format!("email = {}", email));
        }

        query_builder
            .push("WHERE id = $1 RETURNING id, username, email")
            .push_bind(req.id);

        let database_user: DatabaseUser = query_builder
            .build_query_as()
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| match e {
                e if e.is_unique_constraint_violation() => UpdateUserError::Duplicate,
                sqlx::Error::RowNotFound => UpdateUserError::UserNotFound,
                e => UpdateUserError::DatabaseIssues(anyhow!("{e}")),
            })?;

        let user: User = database_user
            .try_into()
            .map_err(|e| UpdateUserError::InvalidUserFromDatabase(anyhow!("{e}")))?;

        tx.commit()
            .await
            .map_err(|e| UpdateUserError::DatabaseIssues(anyhow!("{e}")))?;

        Ok(user)
    }
    //
    // =============================================================================
    //                                  DELETE
    // =============================================================================
    //
    async fn delete_user(&self, req: &DeleteUserRequest) -> Result<(), DeleteUserError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DeleteUserError::DatabaseIssues(anyhow!("{e}")))?;

        let result = query!("DELETE FROM users WHERE id = $1", req.id())
            .execute(&mut *tx)
            .await
            .map_err(|e| DeleteUserError::DatabaseIssues(anyhow!("{e}")))?;

        if result.rows_affected() == 0 {
            return Err(DeleteUserError::NotFound);
        }

        tx.commit()
            .await
            .map_err(|e| DeleteUserError::DatabaseIssues(anyhow!("{e}")))?;

        Ok(())
    }
}
