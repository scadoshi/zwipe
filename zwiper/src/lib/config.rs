use anyhow::Context;
use reqwest::Url;
use std::str::FromStr;

const BACKEND_URL_KEY: &str = "BACKEND_URL";

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub backend_url: Url,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let backend_url_str = std::env::var(BACKEND_URL_KEY)
            .context(format!("missing {} environment variable", BACKEND_URL_KEY))?;
        let backend_url = Url::from_str(&backend_url_str).context(format!(
            "invalid url in {}: {}",
            BACKEND_URL_KEY, backend_url_str
        ))?;
        Ok(Self { backend_url })
    }
}
