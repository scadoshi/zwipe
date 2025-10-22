use crate::outbound::client::{auth::AuthClient, error::ApiError};
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile},
    inbound::http::{
        handlers::deck::update_deck_profile::HttpUpdateDeckProfileBody, routes::update_deck_route,
    },
};

pub trait AuthClientUpdateDeckProfile {
    fn update_deck_profile(
        &self,
        deck_id: &Uuid,
        body: HttpUpdateDeckProfileBody,
        session: &Session,
    ) -> impl Future<Output = Result<DeckProfile, ApiError>> + Send;
}

impl AuthClientUpdateDeckProfile for AuthClient {
    async fn update_deck_profile(
        &self,
        deck_id: &Uuid,
        body: HttpUpdateDeckProfileBody,
        session: &Session,
    ) -> Result<DeckProfile, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&update_deck_route(&deck_id.to_string()));

        let response = self
            .client
            .put(url)
            .json(&body)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let updated: DeckProfile = response.json().await?;
                Ok(updated)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
