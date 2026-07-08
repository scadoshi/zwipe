//! New user registration API client.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::register_route};
use zwipe_core::{domain::auth::models::session::Session, http::contracts::auth::HttpRegisterUser};

/// Trait for registering new user accounts.
#[allow(missing_docs)]
pub trait ClientRegister {
    fn register(
        &self,
        request: HttpRegisterUser,
    ) -> impl Future<Output = Result<Session, ApiError>> + Send;
}

impl ClientRegister for ZwipeClient {
    async fn register(&self, request: HttpRegisterUser) -> Result<Session, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&register_route());
        info!("POST {}", url);
        let response = self.client.post(url).json(&request).send().await?;

        match response.status() {
            StatusCode::CREATED => {
                let new: Session = response.json().await?;
                Ok(new)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
