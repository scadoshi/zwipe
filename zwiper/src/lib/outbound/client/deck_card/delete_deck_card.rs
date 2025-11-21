use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{routes::delete_deck_card_route, ApiError},
};

pub trait ClientDeleteDeckCard {
    fn delete_deck_card(
        &self,
        deck_id: Uuid,
        card_profile_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientDeleteDeckCard for ZwipeClient {
    async fn delete_deck_card(
        &self,
        deck_id: Uuid,
        card_profile_id: Uuid,
        session: &Session,
    ) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&delete_deck_card_route(deck_id, card_profile_id));

        let response = self
            .client
            .delete(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
