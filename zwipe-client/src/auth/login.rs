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
        self.call(Login(stamped(request)), None).await
    }
}

/// `request` with this build's version, and [`ClientPlatform::CURRENT`] unless it already names a platform.
fn stamped(mut request: HttpAuthenticateUser) -> HttpAuthenticateUser {
    request.platform = request.platform.or(Some(ClientPlatform::CURRENT));
    request.client_version = Some(env!("CARGO_PKG_VERSION").to_string());
    request
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_named_platform_is_kept_and_a_missing_one_is_stamped() {
        let mut terminal = HttpAuthenticateUser::new("alice", "pw");
        terminal.platform = Some(ClientPlatform::Terminal);
        assert_eq!(stamped(terminal).platform, Some(ClientPlatform::Terminal));

        let unnamed = stamped(HttpAuthenticateUser::new("alice", "pw"));
        assert_eq!(unnamed.platform, Some(ClientPlatform::CURRENT));
        assert_eq!(
            unnamed.client_version.as_deref(),
            Some(env!("CARGO_PKG_VERSION"))
        );
    }
}
