use crate::outbound::client::ZwipeClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::auth::models::session::Session,
    inbound::http::{routes::delete_deck_route, ApiError},
};

pub trait ClientDeleteDeck {
    fn delete_deck(
        &self,
        deck_id: &Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl ClientDeleteDeck for ZwipeClient {
    async fn delete_deck(&self, deck_id: &Uuid, session: &Session) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&delete_deck_route(&deck_id));

        let response = self
            .client
            .delete(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
