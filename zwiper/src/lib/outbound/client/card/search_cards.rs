use crate::outbound::client::auth::AuthClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::{auth::models::session::Session, card::models::Card},
    inbound::http::{
        handlers::card::search_card::HttpSearchCards, routes::search_cards_route, ApiError,
    },
};

pub trait AuthClientSearchCards {
    fn search_cards(
        &self,
        request: &HttpSearchCards,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ApiError>> + Send;
}

impl AuthClientSearchCards for AuthClient {
    async fn search_cards(
        &self,
        request: &HttpSearchCards,
        session: &Session,
    ) -> Result<Vec<Card>, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&search_cards_route());

        let response = self
            .client
            .get(url)
            .query(request)
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
