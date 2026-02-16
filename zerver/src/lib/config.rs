//! Application configuration loaded from environment variables.
//!
//! Uses [`dotenvy`] to load `.env` files, with all configuration values
//! required at startup. Missing or invalid values cause immediate failure
//! with descriptive error messages.

use crate::domain::auth::models::access_token::JwtSecret;
use anyhow::Context;
use axum::http::HeaderValue;
use std::str::FromStr;
use tracing::Level;

/// Environment variable key for the JWT signing secret.
const JWT_SECRET_KEY: &str = "JWT_SECRET";

/// Environment variable key for the PostgreSQL connection URL.
const DATABASE_URL_KEY: &str = "DATABASE_URL";

/// Environment variable key for the server bind address (e.g., "0.0.0.0:8080").
const BIND_ADDRESS_KEY: &str = "BIND_ADDRESS";

/// Environment variable key for the log level (e.g., "debug", "info", "warn").
const RUST_LOG_KEY: &str = "RUST_LOG";

/// Environment variable key for enabling backtraces ("1" or "full").
const RUST_BACKTRACE_KEY: &str = "RUST_BACKTRACE";

/// Environment variable key for allowed CORS origins (comma-separated).
const ALLOWED_ORIGINS_KEY: &str = "ALLOWED_ORIGINS";

/// Application configuration loaded from environment variables.
///
/// All fields are required and validated at construction time.
pub struct Config {
    /// Secret key for signing and verifying JWT tokens.
    pub jwt_secret: JwtSecret,

    /// PostgreSQL connection URL (e.g., "postgres://user:pass@host/db").
    pub database_url: String,

    /// Address to bind the HTTP server to (e.g., "0.0.0.0:8080").
    pub bind_address: String,

    /// Tracing log level filter.
    pub rust_log: Level,

    /// Backtrace configuration for error reporting.
    pub rust_backtrace: String,

    /// CORS allowed origins for cross-origin requests.
    pub allowed_origins: Vec<HeaderValue>,
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// Attempts to load a `.env` file first, then reads required variables.
    ///
    /// # Errors
    ///
    /// Returns an error if any required variable is missing or invalid.
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let jwt_secret = JwtSecret::new(&env_var_by_key(JWT_SECRET_KEY)?)
            .context("invalid jwt secret from env")?;
        let database_url = env_var_by_key(DATABASE_URL_KEY)?;
        let bind_address = env_var_by_key(BIND_ADDRESS_KEY)?;
        let rust_log = Level::from_str(&env_var_by_key(RUST_LOG_KEY)?)?;
        let rust_backtrace = env_var_by_key(RUST_BACKTRACE_KEY)?;
        let allowed_origins: Vec<HeaderValue> = env_var_by_key(ALLOWED_ORIGINS_KEY)?
            .split(',')
            .map(|x| x.parse())
            .collect::<Result<Vec<HeaderValue>, _>>()?;
        Ok(Self {
            jwt_secret,
            database_url,
            bind_address,
            rust_log,
            rust_backtrace,
            allowed_origins,
        })
    }
}

/// Retrieves an environment variable by key with a descriptive error on failure.
fn env_var_by_key(key: &str) -> anyhow::Result<String> {
    std::env::var(key).context(format!("failed to get variable from env: {}", key))
}
