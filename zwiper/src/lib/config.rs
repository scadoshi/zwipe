//! Frontend application configuration.
//!
//! Loads compile-time environment variables baked in by `build.rs`. Values are
//! fixed per build, so every device running a given binary sees the same ones.

use std::str::FromStr;

use anyhow::Context;
use reqwest::Url;

const BACKEND_URL: &str = env!("BACKEND_URL");
const RUST_LOG: &str = env!("RUST_LOG");
const RUST_BACKTRACE: &str = env!("RUST_BACKTRACE");

/// Frontend application configuration loaded from compile-time environment variables.
///
/// # Build-Time Configuration
///
/// These values are baked into the WASM binary at compile time using `env!()` macro:
/// - `BACKEND_URL`: API server URL (e.g., `http://localhost:3000`)
/// - `RUST_LOG`: Tracing directive (e.g., "info" or "info,zwiper=debug"),
///   parsed by `tracing_subscriber::EnvFilter` at startup.
/// - `RUST_BACKTRACE`: Backtrace configuration ("0", "1", "full")
///
/// # Example `.env` file
///
/// ```text
/// BACKEND_URL=http://localhost:3000
/// RUST_LOG=info,zwiper=debug
/// RUST_BACKTRACE=1
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// Backend API server URL.
    pub backend_url: Url,
    /// Tracing directive (e.g. `"info"` or `"info,zwiper=debug"`). Passed to
    /// `tracing_subscriber::EnvFilter::new`; matches zerver's pattern.
    pub rust_log: String,
    /// Backtrace configuration for debugging.
    pub rust_backtrace: String,
}

impl Config {
    /// Loads configuration from compile-time environment variables.
    ///
    /// # Errors
    ///
    /// Fails if `BACKEND_URL` doesn't parse. `build.rs` checks the variable is
    /// present and well-formed, so this is a build-configuration bug rather
    /// than anything a user can trigger: every copy of a given binary carries
    /// the same value and fails the same way on its first launch.
    pub fn from_env() -> anyhow::Result<Self> {
        Self::from_values(BACKEND_URL, RUST_LOG, RUST_BACKTRACE)
    }

    /// Builds a configuration from raw values, parsing the backend URL.
    fn from_values(
        backend_url: &str,
        rust_log: &str,
        rust_backtrace: &str,
    ) -> anyhow::Result<Self> {
        let backend_url = Url::from_str(backend_url)
            .with_context(|| format!("invalid url in BACKEND_URL: {backend_url}"))?;
        Ok(Self {
            backend_url,
            rust_log: rust_log.to_string(),
            rust_backtrace: rust_backtrace.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_malformed_backend_url_is_an_error() {
        let config = Config::from_values("not a url", "info", "0");
        assert!(config.is_err(), "parsed a malformed url: {config:?}");
    }

    #[test]
    fn a_well_formed_backend_url_parses() {
        let config = Config::from_values("https://api.zwipe.net", "info", "0")
            .expect("a well-formed url parses");
        assert_eq!(config.backend_url.as_str(), "https://api.zwipe.net/");
        assert_eq!(config.rust_log, "info");
    }
}
