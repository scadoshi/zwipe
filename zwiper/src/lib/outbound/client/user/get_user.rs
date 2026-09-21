//! Fetch user profile endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::paths::GET_USER_ROUTE,
};

/// Trait for fetching user profile data.
#[allow(missing_docs)]
pub trait ClientGetUser {
    fn get_user(&self, session: &Session)
    -> impl Future<Output = Result<User, ClientError>> + Send;
}

impl ClientGetUser for ZwipeClient {
    async fn get_user(&self, session: &Session) -> Result<User, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(GET_USER_ROUTE);
        info!("GET {}", url);

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
