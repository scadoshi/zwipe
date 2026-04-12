//! User preferences API client operations.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{routes::preferences_route, ApiError};
use zwipe_core::http::contracts::user::HttpUpdatePreferences;
use zwipe_core::domain::{
    auth::models::session::Session,
    user::preferences::UserPreferences,
};

/// Trait for fetching user display preferences.
#[allow(missing_docs)]
pub trait ClientGetPreferences {
    fn get_preferences(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<UserPreferences, ApiError>> + Send;
}

/// Trait for updating user display preferences.
#[allow(missing_docs)]
pub trait ClientUpdatePreferences {
    fn update_preferences(
        &self,
        request: HttpUpdatePreferences,
        session: &Session,
    ) -> impl Future<Output = Result<UserPreferences, ApiError>> + Send;
}

impl ClientGetPreferences for ZwipeClient {
    async fn get_preferences(&self, session: &Session) -> Result<UserPreferences, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&preferences_route());
        info!("GET {}", url);
        let response = self
            .client
            .get(url)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let prefs: UserPreferences = response.json().await?;
                Ok(prefs)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}

impl ClientUpdatePreferences for ZwipeClient {
    async fn update_preferences(
        &self,
        request: HttpUpdatePreferences,
        session: &Session,
    ) -> Result<UserPreferences, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&preferences_route());
        info!("PUT {} body: {:?}", url, request);
        let response = self
            .client
            .put(url)
            .json(&request)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let prefs: UserPreferences = response.json().await?;
                Ok(prefs)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
