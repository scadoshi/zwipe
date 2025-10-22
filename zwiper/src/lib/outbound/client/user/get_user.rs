use crate::outbound::client::auth::AuthClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::{auth::models::session::Session, user::models::User},
    inbound::http::{routes::get_user_route, ApiError},
};

pub trait AuthClientGetUser {
    fn get_user(&self, session: &Session) -> impl Future<Output = Result<User, ApiError>> + Send;
}

impl AuthClientGetUser for AuthClient {
    async fn get_user(&self, session: &Session) -> Result<User, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_user_route());

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.value.as_str())
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
