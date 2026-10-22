//! Delete user account endpoint.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpDeleteUser;
use zwipe_core::http::endpoints::user::DeleteUser;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Deletes user accounts.
    pub async fn delete_user(
        &self,
        request: HttpDeleteUser,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(DeleteUser(request), Some(session)).await
    }
}
