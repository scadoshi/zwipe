use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{routes::get_sets_route, ApiError},
};

pub trait ClientGetSets {
    fn get_sets(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<String>, ApiError>> + Send;
}

impl ClientGetSets for ZwipeClient {
    async fn get_sets(&self, session: &Session) -> Result<Vec<String>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_sets_route());

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let sets: Vec<String> = response.json().await?;
                Ok(sets)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
