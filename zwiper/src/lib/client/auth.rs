pub mod login;
pub mod logout;
pub mod refresh;
pub mod register;

use crate::config::AppConfig;
use reqwest::Client;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct AuthClient {
    client: Client,
    app_config: AppConfig,
}

impl AuthClient {
    pub fn new() -> Self {
        static CONFIG: OnceLock<AppConfig> = OnceLock::new();
        let app_config = CONFIG
            .get_or_init(|| AppConfig::from_env().expect("failed to initialize app config"))
            .clone();
        Self {
            client: Client::new(),
            app_config,
        }
    }
}
