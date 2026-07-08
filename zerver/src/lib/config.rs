//! Application configuration loaded from environment variables.
//!
//! Uses [`dotenvy`] to load `.env` files, with all configuration values
//! required at startup. Missing or invalid values cause immediate failure
//! with descriptive error messages.

use crate::domain::auth::models::access_token::JwtSecret;
use anyhow::Context;
use axum::http::HeaderValue;

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

/// Environment variable key for the Resend email API key.
const RESEND_API_KEY_KEY: &str = "RESEND_API_KEY";

/// Environment variable key for the outbound email sender address.
const RESEND_EMAIL_FROM_KEY: &str = "RESEND_EMAIL_FROM";

/// Environment variable key for the log file directory.
const LOG_DIR_KEY: &str = "LOG_DIR";

/// Default log directory on production servers.
const LOG_DIR_DEFAULT: &str = "/var/log/zwipe";

/// Environment variable key for the minimum supported client app version.
const MIN_CLIENT_VERSION_KEY: &str = "MIN_CLIENT_VERSION";

/// Default minimum client version — `0.0.0` means the gate is open.
const MIN_CLIENT_VERSION_DEFAULT: &str = "0.0.0";

/// Environment variable key for the public web base URL (email verify/reset links).
const WEB_BASE_URL_KEY: &str = "WEB_BASE_URL";

/// Default public web base URL — the shared `site` constant (debug builds
/// resolve to the local zite dev server, so locally logged verification/reset
/// links open end-to-end); the env var overrides it either way.
const WEB_BASE_URL_DEFAULT: &str = zwipe_core::domain::site::WEB_BASE;

/// Environment variable key for the user-facing support email address.
const SUPPORT_EMAIL_ADDRESS_KEY: &str = "SUPPORT_EMAIL_ADDRESS";

/// Default user-facing support email address.
const SUPPORT_EMAIL_ADDRESS_DEFAULT: &str = zwipe_core::domain::site::SUPPORT_EMAIL;

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

    /// Tracing filter directive(s). Accepts a bare level (`"info"`) or
    /// per-target directives (`"info,sqlx=warn,zwipe=debug"`) — fed to
    /// `tracing_subscriber::EnvFilter`.
    pub rust_log: String,

    /// Backtrace configuration for error reporting.
    pub rust_backtrace: String,

    /// CORS allowed origins for cross-origin requests.
    pub allowed_origins: Vec<HeaderValue>,

    /// Resend API key for sending transactional email.
    pub resend_api_key: String,

    /// Sender address for outbound email (e.g. `"noreply@zwipe.net"`).
    pub resend_from_email: String,

    /// Directory for rolling log files. Defaults to `/var/log/zwipe` if not set.
    pub log_dir: String,

    /// Minimum app version allowed to talk to this server (force-update gate).
    /// Defaults to `0.0.0` (gate open) if not set. Flipping the gate = edit
    /// `.env` on the server + restart zerver; no code deploy.
    pub min_client_version: String,

    /// Public web base URL used to build email verify/reset links. Defaults to
    /// `https://zwipe.net`. A domain change = edit `.env` + restart; no deploy.
    pub web_base_url: String,

    /// User-facing support email shown in transactional emails. Defaults to
    /// `support@zwipe.net`.
    pub support_email_address: String,
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
        let rust_log = env_var_by_key(RUST_LOG_KEY)?;
        let rust_backtrace = env_var_by_key(RUST_BACKTRACE_KEY)?;
        let allowed_origins: Vec<HeaderValue> = env_var_by_key(ALLOWED_ORIGINS_KEY)?
            .split(',')
            .map(|x| x.parse())
            .collect::<Result<Vec<HeaderValue>, _>>()?;
        let resend_api_key = env_var_by_key(RESEND_API_KEY_KEY)?;
        let resend_from_email = env_var_by_key(RESEND_EMAIL_FROM_KEY)?;
        let log_dir = std::env::var(LOG_DIR_KEY).unwrap_or_else(|_| LOG_DIR_DEFAULT.to_string());
        let min_client_version = std::env::var(MIN_CLIENT_VERSION_KEY)
            .unwrap_or_else(|_| MIN_CLIENT_VERSION_DEFAULT.to_string());
        if zwipe_core::version::parse_version(&min_client_version).is_none() {
            anyhow::bail!(
                "invalid {}: {:?} (expected x.y.z)",
                MIN_CLIENT_VERSION_KEY,
                min_client_version
            );
        }
        let web_base_url = std::env::var(WEB_BASE_URL_KEY)
            .unwrap_or_else(|_| WEB_BASE_URL_DEFAULT.to_string())
            .trim_end_matches('/')
            .to_string();
        let support_email_address = std::env::var(SUPPORT_EMAIL_ADDRESS_KEY)
            .unwrap_or_else(|_| SUPPORT_EMAIL_ADDRESS_DEFAULT.to_string());
        Ok(Self {
            jwt_secret,
            database_url,
            bind_address,
            rust_log,
            rust_backtrace,
            allowed_origins,
            resend_api_key,
            resend_from_email,
            log_dir,
            min_client_version,
            web_base_url,
            support_email_address,
        })
    }
}

/// Retrieves an environment variable by key with a descriptive error on failure.
fn env_var_by_key(key: &str) -> anyhow::Result<String> {
    std::env::var(key).context(format!("failed to get variable from env: {}", key))
}
