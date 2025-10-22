use crate::outbound::client::auth::AuthClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{
        handlers::auth::change_password::HttpChangePassword, routes::change_password_route,
        ApiError,
    },
};

pub trait AuthClientChangePassword {
    fn change_password(
        &self,
        request: HttpChangePassword,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl AuthClientChangePassword for AuthClient {
    async fn change_password(
        &self,
        request: HttpChangePassword,
        session: &Session,
    ) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&change_password_route());
        let response = self
            .client
            .put(url)
            .json(&request)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::OK => Ok(()),
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
