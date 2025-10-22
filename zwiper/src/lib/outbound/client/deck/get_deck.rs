use crate::outbound::client::{auth::AuthClient, error::ApiError};
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::Deck},
    inbound::http::routes::get_deck_route,
};

pub trait AuthClientGetDeck {
    fn get_deck(
        &self,
        deck_id: &Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<Deck, ApiError>> + Send;
}

impl AuthClientGetDeck for AuthClient {
    async fn get_deck(&self, deck_id: &Uuid, session: &Session) -> Result<Deck, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_deck_route(&deck_id.to_string()));

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::OK => {
                let deck: Deck = response.json().await?;
                Ok(deck)
            }
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
