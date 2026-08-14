//! Fetch the served keyword-reminder catalog (name → reminder text).

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::{collections::HashMap, future::Future};
use tracing::info;
use zwipe::inbound::http::routes::get_keyword_reminders_route;

/// Trait for fetching the keyword-reminder map. Served so definition fixes
/// land on deploy instead of waiting for an app-store train; the compiled-in
/// table stays as the offline fallback.
#[allow(missing_docs)]
pub trait ClientGetKeywordReminders {
    fn get_keyword_reminders(
        &self,
    ) -> impl Future<Output = Result<HashMap<String, String>, ClientError>> + Send;
}

impl ClientGetKeywordReminders for ZwipeClient {
    async fn get_keyword_reminders(&self) -> Result<HashMap<String, String>, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_keyword_reminders_route());
        info!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let reminders: HashMap<String, String> = response.json().await?;
                Ok(reminders)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
