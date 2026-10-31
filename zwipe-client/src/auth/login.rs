//! User login API client.

use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpAuthenticateUser;
use zwipe_core::http::endpoints::auth::Login;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Authenticates users via the login endpoint. A request with no platform is stamped with [`ClientPlatform::CURRENT`]; one that names its platform, as the terminal client does, keeps it.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn authenticate_user(
        &self,
        request: HttpAuthenticateUser,
    ) -> Result<Session, ClientError> {
        let mut request = request;
        request.platform = request.platform.or(Some(ClientPlatform::CURRENT));
        request.client_version = Some(env!("CARGO_PKG_VERSION").to_string());

        self.call(Login(request), None).await
    }
}
