use std::str::FromStr;

use crate::domain::auth::models::access_token::JwtSecret;
use anyhow::Context;
use axum::http::HeaderValue;
use tracing::Level;

const JWT_SECRET_KEY: &str = "JWT_SECRET";
const DATABASE_URL_KEY: &str = "DATABASE_URL";
const BIND_ADDRESS_KEY: &str = "BIND_ADDRESS";
const RUST_LOG_KEY: &str = "RUST_LOG";
const RUST_BACKTRACE_KEY: &str = "RUST_BACKTRACE";
const ALLOWED_ORIGINS_KEY: &str = "ALLOWED_ORIGINS";

pub struct Config {
    pub jwt_secret: JwtSecret,
    pub database_url: String,
    pub bind_address: String,
    pub rust_log: Level,
    pub rust_backtrace: String,
    pub allowed_origins: Vec<HeaderValue>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let jwt_secret = JwtSecret::new(&env_var_by_key(JWT_SECRET_KEY)?)
            .context("invalid jwt secret from env")?;
        let database_url = env_var_by_key(DATABASE_URL_KEY)?;
        let bind_address = env_var_by_key(BIND_ADDRESS_KEY)?;
        let rust_log = Level::from_str(&env_var_by_key(RUST_LOG_KEY)?)?;
        let rust_backtrace = env_var_by_key(RUST_BACKTRACE_KEY)?;
        let allowed_origins: Vec<HeaderValue> = env_var_by_key(ALLOWED_ORIGINS_KEY)?
            .split(",")
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

fn env_var_by_key(key: &str) -> anyhow::Result<String> {
    std::env::var(key).context(format!("failed to get variable from env: {}", key))
}
