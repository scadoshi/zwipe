//! Change user password endpoint.

use crate::{ClientError, ZwipeClient};
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
        self.call(ChangePassword(request), Some(session)).await
    }
}
