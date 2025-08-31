use anyhow::Context;
use email_address::EmailAddress;
use sqlx::{query, query_as, QueryBuilder};
use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::domain::user::models::{
    CreateUserError, CreateUserRequest, DeleteUserError, DeleteUserRequest, GetUserError,
    GetUserRequest, UpdateUserError, UpdateUserRequest, User, UserName,
};
use crate::domain::user::ports::UserRepository;
use crate::outbound::sqlx::postgres::{IsUniqueConstraintViolation, Postgres};

// ===========
//   db types
// ===========

/// raw database user record
/// (unvalidated data from `PostgreSQL`)
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUser {
    pub id: String,
    pub username: String,
    pub email: String,
}

/// converts database user to validated domain user
impl TryFrom<DatabaseUser> for User {
    type Error = anyhow::Error;

    fn try_from(value: DatabaseUser) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id).context("failed to validate user id")?;
        let username = UserName::new(&value.username).context("failed to validate username")?;
        let email =
            EmailAddress::parse_with_options(&value.email, email_address::Options::default())
                .context("failed to validate email")?;
        Ok(Self {
            id,
            username,
            email,
        })
    }
}

impl UserRepository for Postgres {
    // ========
    //  create
    // ========
    async fn create_user(&self, request: &CreateUserRequest) -> Result<User, CreateUserError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| CreateUserError::Database(e.into()))?;

        let database_user = query_as!(
            DatabaseUser,
            "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id, username, email",
            request.username.to_string(),
            request.email.to_string(),
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if e.is_unique_constraint_violation() {
                return CreateUserError::Duplicate;
            }
            CreateUserError::Database(e.into())
        })?;

        let user: User = database_user
            .try_into()
            .map_err(|e| CreateUserError::InvalidUserFromDatabase(e))?;

        tx.commit()
            .await
            .map_err(|e| CreateUserError::Database(e.into()))?;

        Ok(user)
    }
    // =====
    //  get
    // =====
    async fn get_user(&self, request: &GetUserRequest) -> Result<User, GetUserError> {
        let database_user = query_as!(
            DatabaseUser,
            "SELECT id, username, email FROM users WHERE (id::text = $1 OR username = $1 OR email = $1)",
            request.identifier
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => GetUserError::NotFound,
            e => GetUserError::Database(e.into()),
        })?;

        let user: User = database_user
            .try_into()
            .map_err(|e| GetUserError::InvalidUserFromDatabase(e))?;

        Ok(user)
    }
    // ========
    //  update
    // ========
    async fn update_user(&self, request: &UpdateUserRequest) -> Result<User, UpdateUserError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| UpdateUserError::Database(e.into()))?;

        let mut qb = QueryBuilder::new("UPDATE users SET ");
        let mut sep = qb.separated(", ");
        let mut updates = 0;
        if let Some(username) = &request.username {
            sep.push("username = ")
                .push_bind_unseparated(username.to_string());
            updates += 1;
        }
        if let Some(email) = &request.email {
            sep.push("email = ")
                .push_bind_unseparated(email.to_string());
            updates += 1;
        }
        if updates > 0 {
            let now = chrono::Utc::now().naive_utc();
            sep.push("updated_at =").push_bind_unseparated(now);
        }

        qb.push(" WHERE id = ")
            .push_bind(request.id)
            .push(" RETURNING id, username, email");

        let database_user: DatabaseUser =
            qb.build_query_as()
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| match e {
                    e if e.is_unique_constraint_violation() => UpdateUserError::Duplicate,
                    sqlx::Error::RowNotFound => UpdateUserError::UserNotFound,
                    e => UpdateUserError::Database(e.into()),
                })?;

        let user: User = database_user
            .try_into()
            .map_err(|e| UpdateUserError::InvalidUserFromDatabase(e))?;

        tx.commit()
            .await
            .map_err(|e| UpdateUserError::Database(e.into()))?;

        Ok(user)
    }
    // ========
    //  delete
    // ========
    async fn delete_user(&self, request: &DeleteUserRequest) -> Result<(), DeleteUserError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DeleteUserError::Database(e.into()))?;

        let result = query!("DELETE FROM users WHERE id = $1", request.id())
            .execute(&mut *tx)
            .await
            .map_err(|e| DeleteUserError::Database(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(DeleteUserError::NotFound);
        }

        tx.commit()
            .await
            .map_err(|e| DeleteUserError::Database(e.into()))?;

        Ok(())
    }
}
