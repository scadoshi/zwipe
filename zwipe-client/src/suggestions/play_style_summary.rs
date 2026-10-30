//! Fetch the play style summary Home shows.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;
use zwipe_core::http::endpoints::suggestions::GetPlayStyleSummary;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Reads the user's deck count and color map.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn play_style_summary(
        &self,
        session: &Session,
    ) -> Result<HttpPlayStyleSummary, ClientError> {
        self.call(GetPlayStyleSummary, Some(session)).await
    }
}
