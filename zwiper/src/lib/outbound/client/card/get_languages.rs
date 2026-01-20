use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{routes::get_languages_route, ApiError},
};

pub trait ClientGetLanguages {
    fn get_languages(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<String>, ApiError>> + Send;
}

impl ClientGetLanguages for ZwipeClient {
    async fn get_languages(&self, session: &Session) -> Result<Vec<String>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_languages_route());

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let languages: Vec<String> = response.json().await?;
                Ok(languages)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
