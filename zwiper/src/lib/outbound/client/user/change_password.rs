//! Change user password endpoint.

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpChangePassword, endpoints::user::ChangePassword},
};

impl ZwipeClient {
    /// Updates user passwords.
    pub async fn change_password(
        &self,
        request: HttpChangePassword,
        session: &Session,
    ) -> Result<(), ClientError> {
        let body = serde_json::to_value(&request)?;
        self.call(ChangePassword(body), Some(session)).await
    }
}
