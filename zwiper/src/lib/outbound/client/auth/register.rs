use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{
        handlers::auth::register_user::HttpRegisterUser, routes::register_route, ApiError,
    },
};

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
