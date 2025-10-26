use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::Deck},
    inbound::http::{routes::get_deck_route, ApiError},
};

pub trait ClientGetDeck {
    fn get_deck(
        &self,
        deck_id: &Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<Deck, ApiError>> + Send;
}

impl ClientGetDeck for ZwipeClient {
    async fn get_deck(&self, deck_id: &Uuid, session: &Session) -> Result<Deck, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_deck_route(&deck_id));

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let deck: Deck = response.json().await?;
                Ok(deck)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
