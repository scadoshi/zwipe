//! PostgreSQL database connection and utilities.
//!
//! Provides connection pool configuration with production-ready defaults
//! and helper traits for detecting constraint violations.

use anyhow::Context;

/// PostgreSQL error code for unique constraint violations (SQLSTATE 23505).
const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "23505";

/// PostgreSQL error code for check constraint violations (SQLSTATE 23514).
const CHECK_CONSTRAINT_VIOLATION_CODE: &str = "23514";

/// Connections the pool keeps open while idle.
const MIN_CONNECTIONS: u32 = 2;

/// Most connections the pool opens at once.
const MAX_CONNECTIONS: u32 = 10;

/// How long an unused connection stays open before the pool closes it.
const IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);

/// How long a caller waits for a free connection before the acquire fails.
const ACQUIRE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

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

// == config ==

/// PostgreSQL connection pool with production-ready defaults.
///
/// Wraps [`sqlx::postgres::PgPoolOptions`] with sensible defaults:
/// - 2 minimum connections (keeps pool warm)
/// - 10 maximum connections (prevents overloading the database)
/// - 5 minute idle timeout (releases unused connections)
/// - 5 second acquire timeout (fails fast on pool exhaustion)
#[derive(Debug)]
pub struct PostgresPoolOptions(sqlx::postgres::PgPoolOptions);

impl Default for PostgresPoolOptions {
    fn default() -> Self {
        Self(
            sqlx::postgres::PgPoolOptions::new()
                .min_connections(MIN_CONNECTIONS)
                .max_connections(MAX_CONNECTIONS)
                .idle_timeout(Some(IDLE_TIMEOUT))
                .acquire_timeout(ACQUIRE_TIMEOUT)
                // Pin every connection's session to UTC so NOW(), CURRENT_DATE,
                // and any other clock-derived defaults are deterministic
                // regardless of where the process runs. TIMESTAMP columns
                // store wall-clock values, so this is the only thing keeping
                // them canonically UTC end-to-end.
                .after_connect(|conn, _meta| {
                    Box::pin(async move {
                        use sqlx::Executor;
                        conn.execute("SET TIME ZONE 'UTC'").await?;
                        Ok(())
                    })
                }),
        )
    }
}

impl PostgresPoolOptions {
    /// Creates pool options with optimized settings for production workloads.
    pub fn new() -> Self {
        Self::default()
    }

    /// Establishes a connection pool to the database at the given URL.
    ///
    /// # Errors
    ///
    /// Returns the [`sqlx::Error`] when the pool cannot connect.
    pub async fn connect(self, path: &str) -> Result<sqlx::postgres::PgPool, sqlx::Error> {
        self.0.connect(path).await
    }
}

// == database connection ==

/// PostgreSQL database adapter with connection pooling.
///
/// The primary entry point for database operations. Create one instance
/// at application startup and clone it for each request handler.
#[derive(Debug, Clone)]
pub struct Postgres {
    /// The underlying `SQLx` connection pool.
    pub pool: sqlx::PgPool,
}

impl Postgres {
    /// Creates a new PostgreSQL connection with optimized pool settings.
    ///
    /// The initial connect retries for about a minute: a simultaneous
    /// service restart (unattended-upgrades bouncing Postgres and this
    /// process together) is recoverable, not a config error. The error
    /// omits the connection URL because it carries the database password.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be established within the
    /// retry window.
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        const ATTEMPTS: u32 = 12;
        const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(5);
        // Every attempt but the last is followed by a retry.
        let mut retried_attempts = 1..ATTEMPTS;
        let pool = loop {
            match PostgresPoolOptions::new().connect(path).await {
                Ok(pool) => break pool,
                Err(e) => {
                    let Some(attempt) = retried_attempts.next() else {
                        return Err(e).context(format!(
                            "failed to open the database after {ATTEMPTS} attempts"
                        ));
                    };
                    tracing::warn!(
                        "database connect attempt {attempt}/{ATTEMPTS} failed: {e}; retrying in {}s",
                        RETRY_DELAY.as_secs()
                    );
                    tokio::time::sleep(RETRY_DELAY).await;
                }
            }
        };

        Ok(Self { pool })
    }
}
