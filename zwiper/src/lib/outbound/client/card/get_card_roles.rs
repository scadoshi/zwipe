//! Fetch the card-role catalog.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{domain::card::card_role::CardRoleView, http::endpoints::card::GetCardRoles};

/// Trait for fetching the full card-role catalog (slug, display name, short name).
#[allow(missing_docs)]
pub trait ClientGetCardRoles {
    fn get_card_roles(&self)
    -> impl Future<Output = Result<Vec<CardRoleView>, ClientError>> + Send;
}

impl ClientGetCardRoles for ZwipeClient {
    async fn get_card_roles(&self) -> Result<Vec<CardRoleView>, ClientError> {
        self.call(GetCardRoles, None).await
    }
}
