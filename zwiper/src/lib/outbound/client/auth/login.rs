//! User login API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::{
    domain::auth::models::{platform::ClientPlatform, session::Session},
    http::{contracts::auth::HttpAuthenticateUser, endpoints::auth::Login},
};

impl ZwipeClient {
    /// Authenticates users via the login endpoint.
    pub async fn authenticate_user(
        &self,
        request: HttpAuthenticateUser,
    ) -> Result<Session, ClientError> {
        let mut request = request;
        request.platform = Some(ClientPlatform::CURRENT);
        request.client_version = Some(env!("CARGO_PKG_VERSION").to_string());
        let body = serde_json::to_value(&request)?;
        self.call(Login(body), None).await
    }
}
