use std::future::Future;

use crate::outbound::client::auth::AuthClient;
use thiserror::Error;
use zwipe::inbound::http::routes::logout_route;

#[derive(Debug, Error)]
pub enum LogoutError {
    #[error("thing")]
    Thing,
}

pub trait Logout {
    fn logout(auth_client: &AuthClient) -> impl Future<Output = Result<(), LogoutError>> + Send;
}

impl Logout for AuthClient {
    async fn logout(auth_client: &AuthClient) -> Result<(), LogoutError> {
        let mut url = auth_client.app_config.backend_url.clone();
        url.set_path(&logout_route());

        // x
        todo!()
    }
}
