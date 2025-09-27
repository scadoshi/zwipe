use std::str::FromStr;

use reqwest::Url;

pub struct AppConfig {
    pub backend_url: Url,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            backend_url: Url::from_str("http://localhost:3000")
                .expect("failed to parse default url"),
        }
    }
}
