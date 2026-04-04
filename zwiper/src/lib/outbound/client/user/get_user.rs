//! Fetch user profile endpoint.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::inbound::http::{routes::get_user_route, ApiError};
use zwipe_core::domain::{auth::models::session::Session, user::User};

/// Trait for fetching user profile data.
#[allow(missing_docs)]
pub trait ClientGetUser {
    fn get_user(&self, session: &Session) -> impl Future<Output = Result<User, ApiError>> + Send;
}

impl ClientGetUser for ZwipeClient {
    async fn get_user(&self, session: &Session) -> Result<User, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_user_route());

        let response = self
            .client
            .get(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let user = response.json().await?;
                Ok(user)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
