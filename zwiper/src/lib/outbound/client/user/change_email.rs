use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::{auth::models::session::Session, user::models::User},
    inbound::http::{
        handlers::auth::change_email::HttpChangeEmail, routes::change_email_route, ApiError,
    },
};

pub trait ClientChangeEmail {
    fn change_email(
        &self,
        request: HttpChangeEmail,
        session: &Session,
    ) -> impl Future<Output = Result<User, ApiError>> + Send;
}

impl ClientChangeEmail for ZwipeClient {
    async fn change_email(
        &self,
        request: HttpChangeEmail,
        session: &Session,
    ) -> Result<User, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&change_email_route());
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
