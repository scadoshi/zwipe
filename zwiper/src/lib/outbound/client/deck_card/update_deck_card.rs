use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck_card::DeckCard},
    inbound::http::{
        handlers::deck_card::update_deck_card::HttpUpdateDeckCard, routes::update_deck_card_route,
        ApiError,
    },
};

pub trait ClientUpdateDeckCard {
    fn update_deck_card(
        &self,
        deck_id: Uuid,
        card_profile_id: Uuid,
        request: &HttpUpdateDeckCard,
        session: &Session,
    ) -> impl Future<Output = Result<DeckCard, ApiError>> + Send;
}

impl ClientUpdateDeckCard for ZwipeClient {
    async fn update_deck_card(
        &self,
        deck_id: Uuid,
        card_profile_id: Uuid,
        request: &HttpUpdateDeckCard,
        session: &Session,
    ) -> Result<DeckCard, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&update_deck_card_route(deck_id, card_profile_id));

        let response = self
            .client
            .put(url)
            .json(request)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let updated: DeckCard = response.json().await?;
                Ok(updated)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
