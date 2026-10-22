//! Mark one-time UI hint shown endpoint.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::User;
use zwipe_core::http::contracts::user::HttpMarkHintShown;
use zwipe_core::http::endpoints::user::MarkHintShown;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Marks a one-time UI hint as shown for the authenticated user.
    pub async fn mark_hint_shown(
        &self,
        hint: &str,
        session: &Session,
    ) -> Result<User, ClientError> {
        self.call(
            MarkHintShown(HttpMarkHintShown {
                hint: hint.to_string(),
            }),
            Some(session),
        )
        .await
    }
}
