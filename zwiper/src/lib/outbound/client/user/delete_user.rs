use crate::outbound::client::auth::AuthClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{
        handlers::auth::delete_user::HttpDeleteUser, routes::delete_user_route, ApiError,
    },
};

pub trait AuthClientDeleteUser {
    fn delete_user(
        &self,
        request: HttpDeleteUser,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl AuthClientDeleteUser for AuthClient {
    async fn delete_user(
        &self,
        request: HttpDeleteUser,
        session: &Session,
    ) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&delete_user_route());
        let response = self
            .client
            .delete(url)
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
