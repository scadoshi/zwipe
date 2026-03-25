//! Fetch all keyword abilities.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{routes::get_keywords_route, ApiError},
};

/// Trait for fetching the list of all keyword abilities (flying, trample, etc.).
#[allow(missing_docs)]
pub trait ClientGetKeywords {
    fn get_keywords(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<String>, ApiError>> + Send;
}

impl ClientGetKeywords for ZwipeClient {
    async fn get_keywords(&self, session: &Session) -> Result<Vec<String>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_keywords_route());

        let response = self
            .client
            .get(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let keywords: Vec<String> = response.json().await?;
                Ok(keywords)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
