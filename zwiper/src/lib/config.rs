use std::str::FromStr;

use reqwest::Url;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub backend_url: Url,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            backend_url: Url::from_str("http://127.0.0.1:3000")
                .expect("failed to parse default url"),
        }
    }
}
