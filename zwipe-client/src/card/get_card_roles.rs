//! Fetch the card-role catalog.

use zwipe_core::domain::card::card_role::CardRoleView;
use zwipe_core::http::endpoints::card::GetCardRoles;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the full card-role catalog (slug, display name, short name).
    pub async fn get_card_roles(&self) -> Result<Vec<CardRoleView>, ClientError> {
        self.call(GetCardRoles, None).await
    }
}
