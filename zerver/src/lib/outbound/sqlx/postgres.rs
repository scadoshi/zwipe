//! PostgreSQL database connection and utilities.
//!
//! Provides connection pool configuration with production-ready defaults
//! and helper traits for detecting constraint violations.

use anyhow::Context;

/// PostgreSQL error code for unique constraint violations (SQLSTATE 23505).
const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "23505";

/// PostgreSQL error code for check constraint violations (SQLSTATE 23514).
const CHECK_CONSTRAINT_VIOLATION_CODE: &str = "23514";

/// Extension trait for detecting PostgreSQL constraint violations.
///
/// Allows code to handle specific constraint errors differently from
/// general database errors (e.g., returning 409 Conflict vs 500 Internal).
pub trait IsConstraintViolation {
    /// Returns `true` if this error is a unique constraint violation.
    ///
    /// Useful for detecting duplicate key errors during insert/update.
    fn is_unique_constraint_violation(&self) -> bool;

    /// Returns `true` if this error is a check constraint violation.
    ///
    /// Useful for detecting business rule violations enforced at the database level.
    fn is_check_constraint_violation(&self) -> bool;
}

impl IsConstraintViolation for sqlx::Error {
    fn is_unique_constraint_violation(&self) -> bool {
        if let sqlx::Error::Database(e) = self
            && let Some(code) = e.code()
            && code == UNIQUE_CONSTRAINT_VIOLATION_CODE
        {
            return true;
        }
        false
    }
    fn is_check_constraint_violation(&self) -> bool {
        if let sqlx::Error::Database(e) = self
            && let Some(code) = e.code()
            && code == CHECK_CONSTRAINT_VIOLATION_CODE
        {
            return true;
        }
        false
    }
}

// =========
//   config
// =========

/// PostgreSQL connection pool with production-ready defaults.
///
/// Wraps [`sqlx::postgres::PgPoolOptions`] with sensible defaults:
/// - 2 minimum connections (keeps pool warm)
/// - 10 maximum connections (prevents overloading the database)
/// - 5 minute idle timeout (releases unused connections)
/// - 5 second acquire timeout (fails fast on pool exhaustion)
pub struct PostgresPoolOptions(sqlx::postgres::PgPoolOptions);

impl Default for PostgresPoolOptions {
    fn default() -> Self {
        Self(
            sqlx::postgres::PgPoolOptions::new()
                .min_connections(2)
                .max_connections(10)
                .idle_timeout(Some(std::time::Duration::from_secs(300)))
                .acquire_timeout(std::time::Duration::from_secs(5)),
        )
    }
}

impl PostgresPoolOptions {
    /// Creates pool options with optimized settings for production workloads.
    pub fn new() -> Self {
        Self::default()
    }

    /// Establishes a connection pool to the database at the given URL.
    pub async fn connect(self, path: &str) -> Result<sqlx::postgres::PgPool, sqlx::Error> {
        self.0.connect(path).await
    }
}

// =====================
//  database connection
// =====================

/// PostgreSQL database adapter with connection pooling.
///
/// The primary entry point for database operations. Create one instance
/// at application startup and clone it for each request handler.
#[derive(Debug, Clone)]
pub struct Postgres {
    /// The underlying SQLx connection pool.
    pub pool: sqlx::PgPool,
}

impl Postgres {
    /// Creates a new PostgreSQL connection with optimized pool settings.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be established.
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        let pool = PostgresPoolOptions::new()
            .connect(path)
            .await
            .context(format!("failed to open database at {}", path))?;

        Ok(Self { pool })
    }
}
