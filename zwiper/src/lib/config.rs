//! Frontend application configuration.
//!
//! Loads compile-time environment variables baked in by `build.rs`. Values are
//! fixed per build, so every device running a given binary sees the same ones.

#![allow(clippy::unwrap_used)]

use anyhow::Context;
use reqwest::Url;
use std::str::FromStr;

const BACKEND_URL: &str = env!("BACKEND_URL");
const RUST_LOG: &str = env!("RUST_LOG");
const RUST_BACKTRACE: &str = env!("RUST_BACKTRACE");

/// Frontend application configuration loaded from compile-time environment variables.
///
/// # Build-Time Configuration
///
/// These values are baked into the WASM binary at compile time using `env!()` macro:
/// - `BACKEND_URL`: API server URL (e.g., "http://localhost:3000")
/// - `RUST_LOG`: Tracing directive (e.g., "info" or "info,zwiper=debug") —
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
    /// `tracing_subscriber::EnvFilter::new` — matches zerver's pattern.
    pub rust_log: String,
    /// Backtrace configuration for debugging.
    pub rust_backtrace: String,
}

impl Config {
    /// Loads configuration from compile-time environment variables.
    ///
    /// # Panics
    ///
    /// Panics if `BACKEND_URL` doesn't parse. `build.rs` checks the variable is
    /// present and well-formed, so reaching this is a build-configuration bug
    /// rather than anything a user can trigger: the value is identical in every
    /// copy of a given binary, so a bad one fails on the first launch of any
    /// build rather than for some users and not others.
    pub fn from_env() -> Self {
        let backend_url = Url::from_str(BACKEND_URL)
            .context(format!("invalid url in BACKEND_URL: {}", BACKEND_URL))
            .unwrap();

        let rust_log = RUST_LOG.to_string();

        let rust_backtrace = RUST_BACKTRACE.to_string();

        Self {
            backend_url,
            rust_log,
            rust_backtrace,
        }
    }
}
