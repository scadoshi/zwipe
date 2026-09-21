//! Fetch the card-role catalog.

use crate::outbound::client::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use std::future::Future;
use tracing::info;
use zwipe_core::{domain::card::card_role::CardRoleView, http::paths::GET_CARD_ROLES_ROUTE};

/// Trait for fetching the full card-role catalog (slug, display name, short name).
#[allow(missing_docs)]
pub trait ClientGetCardRoles {
    fn get_card_roles(&self)
    -> impl Future<Output = Result<Vec<CardRoleView>, ClientError>> + Send;
}

impl ClientGetCardRoles for ZwipeClient {
    async fn get_card_roles(&self) -> Result<Vec<CardRoleView>, ClientError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(GET_CARD_ROLES_ROUTE);
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
