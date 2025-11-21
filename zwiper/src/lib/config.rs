use anyhow::Context;
use reqwest::Url;
use std::str::FromStr;
use tracing::Level;

const BACKEND_URL: &str = env!("BACKEND_URL");
const RUST_LOG: &str = env!("RUST_LOG");
const RUST_BACKTRACE: &str = env!("RUST_BACKTRACE");

#[derive(Debug, Clone)]
pub struct Config {
    pub backend_url: Url,
    pub rust_log: Level,
    pub rust_backtrace: String,
}

impl Config {
    pub fn from_env() -> Self {
        let backend_url = Url::from_str(BACKEND_URL)
            .context(format!("invalid url in BACKEND_URL: {}", BACKEND_URL))
            .unwrap();

        let rust_log = tracing::Level::from_str(RUST_LOG)
            .context(format!("invalid trace log in RUST_LOG: {}", RUST_LOG))
            .unwrap();

        let rust_backtrace = RUST_BACKTRACE.to_string();

        Self {
            backend_url,
            rust_log,
            rust_backtrace,
        }
    }
}
