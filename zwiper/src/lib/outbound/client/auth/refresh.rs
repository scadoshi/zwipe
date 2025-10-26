use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{
        handlers::auth::refresh_session::HttpRefreshSession, routes::refresh_session_route,
        ApiError,
    },
};

pub trait ClientRefresh {
    fn refresh(
        &self,
        request: &HttpRefreshSession,
    ) -> impl Future<Output = Result<Session, ApiError>> + Send;
}

impl ClientRefresh for ZwipeClient {
    async fn refresh(&self, request: &HttpRefreshSession) -> Result<Session, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&refresh_session_route());

        let response = self.client.post(url).json(&request).send().await?;

        match response.status() {
            StatusCode::OK => {
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
