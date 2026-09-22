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
        let body = serde_json::to_value(&request)?;
        self.call(DeleteUser(body), Some(session)).await
    }
}
