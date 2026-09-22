//! Mark one-time UI hint shown endpoint.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::{contracts::user::HttpMarkHintShown, endpoints::user::MarkHintShown},
};

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
