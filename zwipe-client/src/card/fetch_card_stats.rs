//! Fetch a card's public numbers by oracle ID.

use uuid::Uuid;
use zwipe_core::http::contracts::card::HttpCardStats;
use zwipe_core::http::endpoints::card::GetCardStats;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches a card's numbers pooled across every user: its decks, MVPs, swipes, the decks it
    /// leads and its commander picks. A card nobody has touched answers zeros, not an error.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_card_stats(&self, oracle_id: Uuid) -> Result<HttpCardStats, ClientError> {
        self.call(GetCardStats(oracle_id), None).await
    }
}
