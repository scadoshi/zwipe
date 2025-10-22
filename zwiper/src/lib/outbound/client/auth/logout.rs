use crate::outbound::client::auth::AuthClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{routes::logout_route, ApiError},
};

pub trait AuthClientLogout {
    fn logout(&self, session: &Session) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl AuthClientLogout for AuthClient {
    async fn logout(&self, session: &Session) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&logout_route());

        let response = self
            .client
            .post(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
