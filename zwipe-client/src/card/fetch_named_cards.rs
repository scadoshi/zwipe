//! Fetch every card by its name's slug.

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::slug::CardSlug;
use zwipe_core::http::endpoints::card::GetNamedCards;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches every card whose name, or front face's name, folds to `slug`, one printing each,
    /// a full-name match first and playable cards before tokens and art cards. Empty when no
    /// card has that slug. Needs no session.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_named_cards(&self, slug: &CardSlug) -> Result<Vec<Card>, ClientError> {
        self.call(GetNamedCards(slug.clone()), None).await
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use reqwest::Url;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    #[tokio::test]
    async fn it_asks_for_the_slug_s_cards_and_reads_the_list() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let read = socket.read(&mut buf).await.unwrap();
            let request = String::from_utf8_lossy(buf.get(..read).unwrap()).into_owned();
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]")
                .await
                .unwrap();
            request
        });
        let client = ZwipeClient::new(Url::parse(&format!("http://{addr}")).unwrap());
        let slug: CardSlug = "fire-ice".parse().unwrap();

        let cards = client.fetch_named_cards(&slug).await.unwrap();

        assert_eq!(cards, Vec::<Card>::new());
        let request = server.await.unwrap();
        assert!(
            request.starts_with("GET /api/card/named/fire-ice "),
            "{request}"
        );
        assert!(
            !request.to_ascii_lowercase().contains("authorization:"),
            "no session: {request}"
        );
    }
}
