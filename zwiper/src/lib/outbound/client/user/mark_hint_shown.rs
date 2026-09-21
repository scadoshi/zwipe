//! Mark one-time UI hint shown endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::{auth::models::session::Session, user::User},
    http::{contracts::user::HttpMarkHintShown, endpoints::user::MarkHintShown},
};

/// Trait for marking a one-time UI hint as shown for the authenticated user.
#[allow(missing_docs)]
pub trait ClientMarkHintShown {
    fn mark_hint_shown(
        &self,
        hint: &str,
        session: &Session,
    ) -> impl Future<Output = Result<User, ClientError>> + Send;
}

impl ClientMarkHintShown for ZwipeClient {
    async fn mark_hint_shown(&self, hint: &str, session: &Session) -> Result<User, ClientError> {
        let body = serde_json::to_value(HttpMarkHintShown {
            hint: hint.to_string(),
        })?;
        self.call(MarkHintShown(body), Some(session)).await
    }
}
