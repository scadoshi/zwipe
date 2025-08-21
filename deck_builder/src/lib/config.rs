use anyhow::Context;
use axum::http::HeaderValue;

use crate::domain::auth::models::jwt::JwtSecret;

const JWT_SECRET_KEY: &str = "JWT_SECRET";
const DATABASE_URL_KEY: &str = "DATABASE_URL";
const BIND_ADDRESS_KEY: &str = "BIND_ADDRESS";
const RUST_LOG_KEY: &str = "RUST_LOG";
const RUST_BACKTRACE_KEY: &str = "RUST_BACKTRACE";
const SCRYFALL_API_BASE_KEY: &str = "SCRYFALL_API_BASE";
const ALLOWED_ORIGINS_KEY: &str = "ALLOWED_ORIGINS";

pub struct Config {
    pub jwt_secret: JwtSecret,
    pub database_url: String,
    pub bind_address: String,
    pub rust_log: String,
    pub rust_backtrace: String,
    pub scryfall_api_base: String,
    pub allowed_origins: Vec<HeaderValue>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().context("Failed to load environment")?;
        let jwt_secret =
            JwtSecret::new(&load_env(JWT_SECRET_KEY)?).context("Invalid jwt secret from env")?;
        let database_url = load_env(DATABASE_URL_KEY)?;
        let bind_address = load_env(BIND_ADDRESS_KEY)?;
        let rust_log = load_env(RUST_LOG_KEY)?;
        let rust_backtrace = load_env(RUST_BACKTRACE_KEY)?;
        let scryfall_api_base = load_env(SCRYFALL_API_BASE_KEY)?;
        let allowed_origins: Vec<HeaderValue> = load_env(ALLOWED_ORIGINS_KEY)?
            .split(",")
            .map(|x| x.parse())
            .collect::<Result<Vec<HeaderValue>, _>>()?;
        Ok(Self {
            jwt_secret,
            database_url,
            bind_address,
            rust_log,
            rust_backtrace,
            scryfall_api_base,
            allowed_origins,
        })
    }
}

fn load_env(key: &str) -> anyhow::Result<String> {
    std::env::var(key).context(format!("Failed to get variable from env: {}", key))
}
