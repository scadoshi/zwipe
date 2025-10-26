use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile},
    inbound::http::{
        handlers::deck::update_deck_profile::HttpUpdateDeckProfile, routes::update_deck_route,
        ApiError,
    },
};

pub trait ClientUpdateDeckProfile {
    fn update_deck_profile(
        &self,
        deck_id: &Uuid,
        body: &HttpUpdateDeckProfile,
        session: &Session,
    ) -> impl Future<Output = Result<DeckProfile, ApiError>> + Send;
}

impl ClientUpdateDeckProfile for ZwipeClient {
    async fn update_deck_profile(
        &self,
        deck_id: &Uuid,
        body: &HttpUpdateDeckProfile,
        session: &Session,
    ) -> Result<DeckProfile, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&update_deck_route(&deck_id));

        let response = self
            .client
            .put(url)
            .json(&body)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let updated: DeckProfile = response.json().await?;
                Ok(updated)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
