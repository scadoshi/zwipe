use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{routes::get_artists_route, ApiError},
};

pub trait ClientGetArtists {
    fn get_artists(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<String>, ApiError>> + Send;
}

impl ClientGetArtists for ZwipeClient {
    async fn get_artists(&self, session: &Session) -> Result<Vec<String>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_artists_route());

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let artists: Vec<String> = response.json().await?;
                Ok(artists)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
