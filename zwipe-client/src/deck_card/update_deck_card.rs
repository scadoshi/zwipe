//! Update a deck card (PATCH: idempotent, absolute quantity).
//!
//! Speaks the PATCH wire shape ([`HttpPatchDeckCard`](zwipe_core::http::contracts::deck_card::HttpPatchDeckCard)): `quantity` sets an
//! absolute value, so retries/replays are harmless. The server's legacy PUT
//! delta route still exists for older shipped clients: migration:
//! `context/plans/patch_idempotent_updates.md`.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_card::DeckCard;
use zwipe_core::http::contracts::deck_card::HttpPatchDeckCard;
use zwipe_core::http::endpoints::deck::UpdateDeckCard;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Updates a card in a deck.
    pub async fn update_deck_card(
        &self,
        deck_id: Uuid,
        scryfall_data_id: Uuid,
        request: &HttpPatchDeckCard,
        session: &Session,
    ) -> Result<DeckCard, ClientError> {
        self.call(
            UpdateDeckCard(deck_id, scryfall_data_id, request.clone()),
            Some(session),
        )
        .await
    }
}
