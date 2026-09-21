//! User login API client.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::auth::models::{platform::ClientPlatform, session::Session},
    http::{contracts::auth::HttpAuthenticateUser, endpoints::auth::Login},
};

/// Trait for authenticating users via the login endpoint.
#[allow(missing_docs)]
pub trait ClientLogin {
    fn authenticate_user(
        &self,
        request: HttpAuthenticateUser,
    ) -> impl Future<Output = Result<Session, ClientError>> + Send;
}

impl ClientLogin for ZwipeClient {
    async fn authenticate_user(
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
