//! User preferences API client operations.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::{auth::models::session::Session, user::preferences::UserPreferences},
    http::{
        contracts::user::HttpUpdatePreferences,
        endpoints::user::{GetPreferences, UpdatePreferences},
    },
};

impl ZwipeClient {
    /// Fetches user display preferences.
    pub async fn get_preferences(&self, session: &Session) -> Result<UserPreferences, ClientError> {
        self.call(GetPreferences, Some(session)).await
    }

    /// Updates user display preferences.
    pub async fn update_preferences(
        &self,
        request: HttpUpdatePreferences,
        session: &Session,
    ) -> Result<UserPreferences, ClientError> {
        self.call(UpdatePreferences(request), Some(session)).await
    }
}
