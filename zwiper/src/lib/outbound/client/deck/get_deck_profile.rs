use crate::outbound::client::auth::AuthClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile},
    inbound::http::{routes::get_deck_profile_route, ApiError},
};

pub trait AuthClientGetDeckProfile {
    fn get_deck_profile(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<DeckProfile, ApiError>> + Send;
}

impl AuthClientGetDeckProfile for AuthClient {
    async fn get_deck_profile(
        &self,
        deck_id: Uuid,
        session: &Session,
    ) -> Result<DeckProfile, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_deck_profile_route(&deck_id));

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let deck_profile: DeckProfile = response.json().await?;
                Ok(deck_profile)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
