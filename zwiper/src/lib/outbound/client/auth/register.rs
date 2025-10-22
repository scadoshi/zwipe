use crate::outbound::client::{auth::AuthClient, error::ApiError};
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{handlers::auth::register_user::HttpRegisterUser, routes::register_route},
};

pub trait AuthClientRegister {
    fn register(
        &self,
        request: HttpRegisterUser,
    ) -> impl Future<Output = Result<Session, ApiError>> + Send;
}

impl AuthClientRegister for AuthClient {
    async fn register(&self, request: HttpRegisterUser) -> Result<Session, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&register_route());
        let response = self.client.post(url).json(&request).send().await?;

        let status = response.status();

        match status {
            StatusCode::CREATED => {
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
