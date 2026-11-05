//! User settings API client operations.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::settings::UserSettings;
use zwipe_core::http::contracts::user::HttpUpdateSettings;
use zwipe_core::http::endpoints::user::{GetSettings, UpdateSettings};

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches user display settings.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_settings(&self, session: &Session) -> Result<UserSettings, ClientError> {
        self.call(GetSettings, Some(session)).await
    }

    /// Updates user display settings.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn update_settings(
        &self,
        request: HttpUpdateSettings,
        session: &Session,
    ) -> Result<UserSettings, ClientError> {
        self.call(UpdateSettings(request), Some(session)).await
    }
}
