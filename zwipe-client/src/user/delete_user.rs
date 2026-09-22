//! Delete user account endpoint.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::auth::models::session::Session,
    http::{contracts::auth::HttpDeleteUser, endpoints::user::DeleteUser},
};

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
