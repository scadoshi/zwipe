// =============================================================================
// IMPORTS
// =============================================================================

use anyhow::{anyhow, Context};
use sqlx::{query_as, PgPool, Transaction};
use tracing::info;

use crate::domain::user::models::{UserAuthenticationError, UserRegistrationError, UserCreationRequest};
use crate::outbound::sqlx::user::{DatabaseUser, DatabaseUserWithPasswordHash};

// =============================================================================
// CONSTANTS & HELPERS
// =============================================================================

/// PostgreSQL error code for unique constraint violations
const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "23505";

trait IsUniqueConstraintViolation {
    fn is_unique_constraint_violation(&self) -> bool;
}

impl IsUniqueConstraintViolation for sqlx::Error {
    fn is_unique_constraint_violation(&self) -> bool {
        if let sqlx::Error::Database(e) = self {
            if let Some(code) = e.code() {
                if code == UNIQUE_CONSTRAINT_VIOLATION_CODE {
                    return true;
                }
            }
        }
        false
    }
}

// =============================================================================
// CONFIGURATION TYPES
// =============================================================================

/// PostgreSQL connection pool with production-ready defaults
pub struct PostgresPoolOptions(sqlx::postgres::PgPoolOptions);

impl PostgresPoolOptions {
    /// Creates pool options with optimized settings for production workloads
    pub fn new() -> Self {
        Self(
            sqlx::postgres::PgPoolOptions::new()
                .min_connections(2)
                .max_connections(10)
                .idle_timeout(Some(std::time::Duration::from_secs(300)))
                .acquire_timeout(std::time::Duration::from_secs(5)),
        )
    }

    pub async fn connect(self, path: &str) -> Result<sqlx::postgres::PgPool, sqlx::Error> {
        self.0.connect(path).await
    }
}

// =============================================================================
// DATABASE CONNECTION
// =============================================================================

/// PostgreSQL database adapter with connection pooling
#[derive(Debug, Clone)]
pub struct Postgres {
    pub pool: sqlx::PgPool,
}

impl Postgres {
    /// Creates new PostgreSQL connection with optimized pool settings
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        info!("Generating connection pool to database");

        let pool = PostgresPoolOptions::new()
            .connect(path)
            .await
            .with_context(|| format!("Failed to open database at {}", path))?;

        Ok(Self { pool })
    }

    /// Saves user to database
    pub async fn save_user(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        req: &UserCreationRequest,
    ) -> Result<DatabaseUser, UserRegistrationError> {
        query_as!(
            DatabaseUser,
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email",
            req.username.to_string(),
            req.email.to_string(),
            req.password_hash.to_string()
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| {
            if e.is_unique_constraint_violation() {
                return UserRegistrationError::Duplicate;
            } 
            UserRegistrationError::DatabaseIssues(anyhow!("{}", e))
        })
    }

    /// Gets user with password hash from database for user with matching username or email
    pub async fn get_user_with_password_hash_with_username_or_email(
        &self,
        pool: &PgPool,
        username_or_email: &str,
    ) -> Result<DatabaseUserWithPasswordHash, UserAuthenticationError> {
        query_as!(
            DatabaseUserWithPasswordHash,
            "SELECT id, username, email, password_hash FROM users WHERE username = $1 OR email = $1",
            username_or_email
        )
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => UserAuthenticationError::UserNotFound,
            e => UserAuthenticationError::DatabaseIssues(anyhow!("{e}")),
        })
    }
}
