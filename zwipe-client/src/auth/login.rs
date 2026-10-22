//! User login API client.

use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpAuthenticateUser;
use zwipe_core::http::endpoints::auth::Login;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Authenticates users via the login endpoint.
    pub async fn authenticate_user(
        &self,
        request: HttpAuthenticateUser,
    ) -> Result<Session, ClientError> {
        let mut request = request;
        request.platform = Some(ClientPlatform::CURRENT);
        request.client_version = Some(env!("CARGO_PKG_VERSION").to_string());

        self.call(Login(request), None).await
    }
}
