//! Fetch the play style summary Home shows.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;
use zwipe_core::http::endpoints::suggestions::GetPlayStyleSummary;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Reads the user's deck count and most built color identities.
    pub async fn play_style_summary(
        &self,
        session: &Session,
    ) -> Result<HttpPlayStyleSummary, ClientError> {
        self.call(GetPlayStyleSummary, Some(session)).await
    }
}
