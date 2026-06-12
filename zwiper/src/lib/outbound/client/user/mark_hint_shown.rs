//! Mark one-time UI hint shown endpoint.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{routes::mark_hint_shown_route, ApiError};
use zwipe_core::domain::{auth::models::session::Session, user::User};
use zwipe_core::http::contracts::user::HttpMarkHintShown;

/// Trait for marking a one-time UI hint as shown for the authenticated user.
#[allow(missing_docs)]
pub trait ClientMarkHintShown {
    fn mark_hint_shown(
        &self,
        hint: &str,
        session: &Session,
    ) -> impl Future<Output = Result<User, ApiError>> + Send;
}

impl ClientMarkHintShown for ZwipeClient {
    async fn mark_hint_shown(&self, hint: &str, session: &Session) -> Result<User, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&mark_hint_shown_route());
        info!("PUT {}", url);

        let body = HttpMarkHintShown {
            hint: hint.to_string(),
        };

        let response = self
            .client
            .put(url)
            .bearer_auth(&*session.access_token.value)
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let user = response.json().await?;
                Ok(user)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
