pub mod auth;
pub mod card;
pub mod deck;
pub mod deck_card;
pub mod user;

use crate::config::Config;
use reqwest::Client;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct ZwipeClient {
    pub client: Client,
    pub app_config: Config,
}

impl Default for ZwipeClient {
    fn default() -> Self {
        static CONFIG: OnceLock<Config> = OnceLock::new();
        let app_config = CONFIG.get_or_init(Config::from_env).clone();
        Self {
            client: Client::new(),
            app_config,
        }
    }
}

impl ZwipeClient {
    pub fn new() -> Self {
        Self::default()
    }
}
