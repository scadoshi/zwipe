//! Fetch a single card by ID.

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::endpoints::card::GetCard;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches a single card by its Scryfall data ID, once a session: a card this client, or any
    /// clone of it, has fetched before comes from its cache.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_card(&self, scryfall_data_id: Uuid) -> Result<Card, ClientError> {
        if let Some(card) = self
            .cards
            .lock()
            .ok()
            .and_then(|cards| cards.get(&scryfall_data_id).cloned())
        {
            return Ok(card);
        }
        let card: Card = self.call(GetCard(scryfall_data_id), None).await?;
        if let Ok(mut cards) = self.cards.lock() {
            cards.insert(scryfall_data_id, card.clone());
        }
        Ok(card)
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use reqwest::Url;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    /// A real card, as the API serves it.
    const CARD_JSON: &str = include_str!("../../../zwipe-ui/src/test_support/card.json");

    #[tokio::test]
    async fn a_card_is_fetched_once_a_session_across_the_client_s_clones() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let served = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&served);
        tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buf = vec![0_u8; 4096];
                let _ = socket.read(&mut buf).await;
                count.fetch_add(1, Ordering::SeqCst);
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{CARD_JSON}",
                    CARD_JSON.len()
                );
                socket.write_all(reply.as_bytes()).await.unwrap();
            }
        });
        let client = ZwipeClient::new(Url::parse(&format!("http://{addr}")).unwrap());
        let id: Card = serde_json::from_str(CARD_JSON).unwrap();
        let id = id.scryfall_data.id;
        let first = client.fetch_card(id).await.unwrap();
        let second = client.clone().fetch_card(id).await.unwrap();
        assert_eq!(first.scryfall_data.id, second.scryfall_data.id);
        assert_eq!(
            served.load(Ordering::SeqCst),
            1,
            "the second comes from the client's cache"
        );
    }
}
