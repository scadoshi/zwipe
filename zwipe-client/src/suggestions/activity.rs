//! Fetch the data behind the Activity screen.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::HttpActivity;
use zwipe_core::http::endpoints::suggestions::GetActivity;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Reads the user's daily activity, swipe outcomes, keep rate by role
    /// and hard passes.
    pub async fn activity(&self, session: &Session) -> Result<HttpActivity, ClientError> {
        self.call(GetActivity, Some(session)).await
    }
}
