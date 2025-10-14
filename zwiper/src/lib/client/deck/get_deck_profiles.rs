use crate::client::auth::AuthClient;
use std::future::Future;
use thiserror::Error;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile},
    inbound::http::routes::get_deck_profiles_route,
};

#[derive(Debug, Error)]
pub enum GetDeckProfilesError {
    #[error("network error")]
    Network(reqwest::Error),
    #[error("something went wrong")]
    SomethingWentWrong,
}

impl From<reqwest::Error> for GetDeckProfilesError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value)
    }
}

impl From<serde_json::Error> for GetDeckProfilesError {
    fn from(_value: serde_json::Error) -> Self {
        Self::SomethingWentWrong
    }
}

pub trait GetDecks {
    fn get_deck_profiles(
        &self,
        session: Session,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, GetDeckProfilesError>> + Send;
}

impl GetDecks for AuthClient {
    async fn get_deck_profiles(
        &self,
        session: Session,
    ) -> Result<Vec<DeckProfile>, GetDeckProfilesError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_deck_profiles_route());

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.jwt.as_str())
            .send()
            .await?;

        let deck_profiles: Vec<DeckProfile> = response.json().await?;

        Ok(deck_profiles)
    }
}
