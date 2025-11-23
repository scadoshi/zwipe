use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{search_card::card_filter::CardFilter, Card},
    },
    inbound::http::{routes::search_cards_route, ApiError},
};

pub trait ClientSearchCards {
    fn search_cards(
        &self,
        card_filter: &CardFilter,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ApiError>> + Send;
}

impl ClientSearchCards for ZwipeClient {
    async fn search_cards(
        &self,
        card_filter: &CardFilter,
        session: &Session,
    ) -> Result<Vec<Card>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&search_cards_route());

        let response = self
            .client
            .post(url)
            .json(card_filter)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let cards: Vec<Card> = response.json().await?;
                Ok(cards)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
