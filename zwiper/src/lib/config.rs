use anyhow::Context;
use reqwest::Url;
use std::str::FromStr;

const BACKEND_URL: &str = env!("BACKEND_URL");

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub backend_url: Url,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let backend_url = Url::from_str(&BACKEND_URL)
            .context(format!("invalid url in BACKEND_URL: {}", BACKEND_URL))?;
        Ok(Self { backend_url })
    }
}
