//! Fetch the ranked commander suggestions.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::HttpCommanderSuggestions;
use zwipe_core::http::endpoints::suggestions::GetCommanderSuggestions;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Reads the user's commander suggestions, at most `limit` of them.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn commander_suggestions(
        &self,
        limit: u32,
        session: &Session,
    ) -> Result<HttpCommanderSuggestions, ClientError> {
        self.call(GetCommanderSuggestions(limit), Some(session))
            .await
    }
}
