use crate::outbound::client::auth::AuthClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck_card::DeckCard},
    inbound::http::{
        handlers::deck_card::create_deck_card::HttpCreateDeckCard, routes::create_deck_card_route,
        ApiError,
    },
};

pub trait AuthClientCreateDeckCard {
    fn create_deck_card(
        &self,
        deck_id: &Uuid,
        request: &HttpCreateDeckCard,
        session: &Session,
    ) -> impl Future<Output = Result<DeckCard, ApiError>> + Send;
}

impl AuthClientCreateDeckCard for AuthClient {
    async fn create_deck_card(
        &self,
        deck_id: &Uuid,
        request: &HttpCreateDeckCard,
        session: &Session,
    ) -> Result<DeckCard, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&create_deck_card_route(&deck_id.to_string()));

        let response = self
            .client
            .post(url)
            .json(request)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let new: DeckCard = response.json().await?;
                Ok(new)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
