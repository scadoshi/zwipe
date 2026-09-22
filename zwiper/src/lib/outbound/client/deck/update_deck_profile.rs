//! Update deck profile metadata.

use crate::outbound::client::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::{auth::models::session::Session, deck::deck_profile::DeckProfile},
    http::{contracts::deck::HttpUpdateDeckProfile, endpoints::deck::UpdateDeckProfile},
};

impl ZwipeClient {
    /// Updates deck profile metadata.
    pub async fn update_deck_profile(
        &self,
        deck_id: Uuid,
        body: &HttpUpdateDeckProfile,
        session: &Session,
    ) -> Result<DeckProfile, ClientError> {
        let body = serde_json::to_value(body)?;
        self.call(UpdateDeckProfile(deck_id, body), Some(session))
            .await
    }
}
