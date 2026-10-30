//! Fetch the card-role catalog.

use zwipe_core::domain::card::card_role::CardRoleView;
use zwipe_core::http::endpoints::card::GetCardRoles;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the full card-role catalog (slug, display name, short name).
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_card_roles(&self) -> Result<Vec<CardRoleView>, ClientError> {
        self.call(GetCardRoles, None).await
    }
}
