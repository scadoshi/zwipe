use anyhow::Context;

/// postgresql error code for unique constraint violations
const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "23505";
const CHECK_CONSTRAINT_VIOLATION_CODE: &str = "23514";

pub trait IsConstraintViolation {
    fn is_unique_constraint_violation(&self) -> bool;
    fn is_check_constraint_violation(&self) -> bool;
}

impl IsConstraintViolation for sqlx::Error {
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
    fn is_check_constraint_violation(&self) -> bool {
        if let sqlx::Error::Database(e) = self {
            if let Some(code) = e.code() {
                if code == CHECK_CONSTRAINT_VIOLATION_CODE {
                    return true;
                }
            }
        }
        false
    }
}

// =========
//   config
// =========

/// postgresql connection pool with production-ready defaults
pub struct PostgresPoolOptions(sqlx::postgres::PgPoolOptions);

impl PostgresPoolOptions {
    /// creates pool options with optimized settings for production workloads
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

// =====================
//  database connection
// =====================

/// postgresql database adapter with connection pooling
#[derive(Debug, Clone)]
pub struct Postgres {
    pub pool: sqlx::PgPool,
}

impl Postgres {
    /// creates new postgresql connection with optimized pool settings
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        let pool = PostgresPoolOptions::new()
            .connect(path)
            .await
            .context(format!("failed to open database at {}", path))?;

        Ok(Self { pool })
    }
}
