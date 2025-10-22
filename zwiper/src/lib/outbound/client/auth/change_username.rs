use crate::outbound::client::{auth::AuthClient, error::ApiError};
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::{auth::models::session::Session, user::models::User},
    inbound::http::{
        handlers::auth::change_username::HttpChangeUsername, routes::change_username_route,
    },
};

pub trait AuthClientChangeUsername {
    fn change_username(
        &self,
        request: HttpChangeUsername,
        session: &Session,
    ) -> impl Future<Output = Result<User, ApiError>> + Send;
}

impl AuthClientChangeUsername for AuthClient {
    async fn change_username(
        &self,
        request: HttpChangeUsername,
        session: &Session,
    ) -> Result<User, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&change_username_route());
        let response = self
            .client
            .put(url)
            .json(&request)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let updated: User = response.json().await?;
                Ok(updated)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
