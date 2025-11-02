use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{routes::get_card_types_route, ApiError},
};

pub trait ClientGetCardTypes {
    fn get_card_types(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<String>, ApiError>> + Send;
}

impl ClientGetCardTypes for ZwipeClient {
    async fn get_card_types(&self, session: &Session) -> Result<Vec<String>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_card_types_route());

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let all_types: Vec<String> = response.json().await?;
                Ok(all_types)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
