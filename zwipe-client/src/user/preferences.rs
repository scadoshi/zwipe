//! User preferences API client operations.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::preferences::UserPreferences;
use zwipe_core::http::contracts::user::HttpUpdatePreferences;
use zwipe_core::http::endpoints::user::{GetPreferences, UpdatePreferences};

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches user display preferences.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_preferences(
        &self,
        session: &Session,
    ) -> Result<UserPreferences, ClientError> {
        self.call(GetPreferences, Some(session)).await
    }

    /// Updates user display preferences.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn update_preferences(
        &self,
        request: HttpUpdatePreferences,
        session: &Session,
    ) -> Result<UserPreferences, ClientError> {
        self.call(UpdatePreferences(request), Some(session)).await
    }
}
