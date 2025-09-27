use zwipe::{
    domain::{
        auth::models::{
            AuthenticateUser, InvalidAuthenticateUser, InvalidRawRegisterUser, RawRegisterUser,
        },
        user::models::User,
    },
    inbound::http::handlers::auth::{HttpAuthenticateUser, HttpRegisterUser},
};

use reqwest::{Client, Url};

use crate::config::AppConfig;

pub struct AuthClient {
    client: Client,
    app_config: AppConfig,
}

impl Default for AuthClient {
    fn default() -> Self {
        let app_config = AppConfig::default();
        let client = Client::new();
        Self { client, app_config }
    }
}

pub fn validate_register_user(
    username: &str,
    email: &str,
    password: &str,
) -> Result<HttpRegisterUser, InvalidRawRegisterUser> {
    RawRegisterUser::new(username, email, password).map(|r| r.into())
}

pub async fn register_user(
    request: HttpRegisterUser,
    auth_client: &AuthClient,
) -> Result<User, String> {
    let response = auth_client
        .client
        .post(auth_client.app_config.backend_url.clone())
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request).map_err(|e| e.to_string())?)
        .send()
        .await;

    todo!()
}

pub fn validate_authenticate_user(
    identifier: &str,
    password: &str,
) -> Result<HttpAuthenticateUser, InvalidAuthenticateUser> {
    AuthenticateUser::new(identifier, password).map(|r| r.into())
}

pub async fn authenticate_user(request: HttpAuthenticateUser, auth_client: &AuthClient) {
    todo!()
}
