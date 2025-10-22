use crate::outbound::client::{auth::AuthClient, error::ApiError};
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{domain::auth::models::session::Session, inbound::http::routes::delete_deck_route};

pub trait AuthClientDeleteDeck {
    fn delete_deck(
        &self,
        deck_id: &Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ApiError>> + Send;
}

impl AuthClientDeleteDeck for AuthClient {
    async fn delete_deck(&self, deck_id: &Uuid, session: &Session) -> Result<(), ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&delete_deck_route(&deck_id.to_string()));

        let response = self
            .client
            .delete(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        let status = response.status();

        match status {
            StatusCode::NO_CONTENT => Ok(()),
            _ => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
