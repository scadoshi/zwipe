pub mod login;
pub mod logout;
pub mod refresh;
pub mod register;

use crate::config::Config;
use reqwest::Client;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct AuthClient {
    client: Client,
    app_config: Config,
}

impl AuthClient {
    pub fn new() -> Self {
        static CONFIG: OnceLock<Config> = OnceLock::new();
        let app_config = CONFIG.get_or_init(|| Config::from_env()).clone();
        Self {
            client: Client::new(),
            app_config,
        }
    }
}
