use crate::outbound::client::{auth::AuthClient, error::ApiError};
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile},
    inbound::http::routes::get_deck_profiles_route,
};

pub trait AuthClientGetDecks {
    fn get_deck_profiles(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, ApiError>> + Send;
}

impl AuthClientGetDecks for AuthClient {
    async fn get_deck_profiles(&self, session: &Session) -> Result<Vec<DeckProfile>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_deck_profiles_route());

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let deck_profiles: Vec<DeckProfile> = response.json().await?;
                Ok(deck_profiles)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
