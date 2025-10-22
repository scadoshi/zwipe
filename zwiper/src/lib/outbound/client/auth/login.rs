use crate::outbound::client::{auth::AuthClient, error::ApiError};
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{handlers::auth::authenticate_user::HttpAuthenticateUser, routes::login_route},
};

pub trait AuthClientLogin {
    fn authenticate_user(
        &self,
        request: HttpAuthenticateUser,
    ) -> impl Future<Output = Result<Session, ApiError>> + Send;
}

impl AuthClientLogin for AuthClient {
    async fn authenticate_user(&self, request: HttpAuthenticateUser) -> Result<Session, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&login_route());
        let response = self.client.post(url).json(&request).send().await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let new: Session = response.json().await?;
                Ok(new)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
