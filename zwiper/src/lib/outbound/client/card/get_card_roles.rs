//! Fetch the card-role catalog.

use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe::inbound::http::{ApiError, routes::get_card_roles_route};
use zwipe_core::domain::card::card_role::CardRoleView;

/// Trait for fetching the full card-role catalog (slug, display name, short name).
#[allow(missing_docs)]
pub trait ClientGetCardRoles {
    fn get_card_roles(&self) -> impl Future<Output = Result<Vec<CardRoleView>, ApiError>> + Send;
}

impl ClientGetCardRoles for ZwipeClient {
    async fn get_card_roles(&self) -> Result<Vec<CardRoleView>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_card_roles_route());
        info!("GET {}", url);

        let response = self.client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                let roles: Vec<CardRoleView> = response.json().await?;
                Ok(roles)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
