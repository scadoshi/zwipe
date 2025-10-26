use crate::outbound::client::auth::AuthClient;
use reqwest::StatusCode;
use std::future::Future;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, card::models::Card},
    inbound::http::{routes::get_card_route, ApiError},
};

pub trait AuthClientGetCard {
    fn get_card(
        &self,
        card_profile_id: &Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<Card, ApiError>> + Send;
}

impl AuthClientGetCard for AuthClient {
    async fn get_card(&self, card_profile_id: &Uuid, session: &Session) -> Result<Card, ApiError> {
        let mut url = self.app_config.backend_url.clone();
        url.set_path(&get_card_route(&card_profile_id));

        let response = self
            .client
            .get(url)
            .bearer_auth(session.access_token.value.as_str())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let card: Card = response.json().await?;
                Ok(card)
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}
