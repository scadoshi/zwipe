use crate::outbound::client::auth::AuthClient;
use reqwest::StatusCode;
use std::future::Future;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile},
    inbound::http::{
        handlers::deck::create_deck_profile::HttpCreateDeckProfile, routes::create_deck_route,
        ApiError,
    },
};

pub trait AuthClientCreateDeck {
    fn create_deck_profile(
        &self,
        request: &HttpCreateDeckProfile,
        session: &Session,
    ) -> impl Future<Output = Result<DeckProfile, ApiError>> + Send;
}

impl AuthClientCreateDeck for AuthClient {
    async fn create_deck_profile(
        &self,
        request: &HttpCreateDeckProfile,
        session: &Session,
    ) -> Result<DeckProfile, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&create_deck_route());

        let response = self
            .client
            .post(url)
            .json(request)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let new: DeckProfile = response.json().await?;
                Ok(new)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
