//! HTTP API client for backend communication.
//!
//! Provides the [`ZwipeClient`] struct and sub-modules for each API domain:
//! authentication, cards, decks, deck cards, and user management.

/// Authentication endpoints (login, logout, register, refresh).
pub mod auth;
/// The one place an API request is built, sent and decoded.
pub mod call;
/// Card data endpoints (search, get, types, artists, sets, languages).
pub mod card;
/// Changelog endpoint (release history).
pub mod changelog;
/// Deck CRUD operations.
pub mod deck;
/// Deck-card relationship operations (add/remove cards from decks).
pub mod deck_card;
/// Client-side error type (transport failures + wire-error decoding).
pub mod error;
/// User metrics operations.
pub mod metrics;
/// Suggested commanders and their dismissals.
pub mod suggestions;
/// User profile operations.
pub mod user;
/// Client version gate operations.
pub mod version;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

pub use error::ClientError;
use reqwest::{Client, Url};

/// HTTP client for the ZWIPE backend API.
///
/// The base URL arrives at construction: the crate reads no environment and
/// owns no default, so each app supplies the value the way it already sources
/// it (zwiper from its `.env`, zite from a const).
#[derive(Debug, Clone)]
pub struct ZwipeClient {
    /// The underlying HTTP client.
    pub client: Client,
    /// Scheme, host and port every request path is joined onto.
    pub base_url: Url,
}

/// Longest a request may take end to end. A stalled socket otherwise holds
/// the refresh single-flight lock, and with it every authenticated call,
/// until the OS gives up on it.
#[cfg(not(target_arch = "wasm32"))]
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

impl ZwipeClient {
    /// Builds a client pointed at `base_url`.
    pub fn new(base_url: Url) -> Self {
        Self {
            client: build_client(),
            base_url,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[expect(
    clippy::expect_used,
    reason = "a builder with only a timeout set has nothing to fail on"
)]
fn build_client() -> Client {
    Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .expect("reqwest client with only a timeout set always builds")
}

/// The browser's fetch has no client-wide timeout; the page's own navigation
/// and the server's 30-second timeout bound a request there.
#[cfg(target_arch = "wasm32")]
fn build_client() -> Client {
    Client::new()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::*;

    #[tokio::test]
    async fn native_client_asks_for_gzip_and_brotli() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let read = socket.read(&mut buf).await.unwrap();
            socket
                .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
                .await
                .unwrap();
            String::from_utf8_lossy(&buf[..read]).to_lowercase()
        });

        build_client()
            .get(format!("http://{addr}/"))
            .send()
            .await
            .unwrap();
        let request = server.await.unwrap();
        let accept = request
            .lines()
            .find_map(|line| line.strip_prefix("accept-encoding:"))
            .unwrap_or_default();

        assert!(accept.contains("gzip"), "request {request:?}");
        assert!(accept.contains("br"), "request {request:?}");
    }

    #[tokio::test]
    async fn a_call_within_a_timeout_gives_up_on_a_server_that_never_answers() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        // Accepts, reads the request, and never answers: a dead mobile link.
        let _server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 4096];
            let _ = socket.read(&mut buf).await;
            tokio::time::sleep(Duration::from_secs(60)).await;
        });
        let client = ZwipeClient::new(Url::parse(&format!("http://{addr}")).unwrap());

        let started = std::time::Instant::now();
        let result = client
            .call_within(
                zwipe_core::http::endpoints::card::GetPrintings(uuid::Uuid::nil()),
                None,
                Duration::from_millis(300),
            )
            .await;

        assert!(
            matches!(result, Err(ClientError::Network(_))),
            "a timed-out call is a network error: {result:?}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "gave up after {:?}",
            started.elapsed()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_deck_card_search_gives_up_on_a_server_that_never_answers() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        // Accepts, reads the request, and never answers.
        let _server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 8192];
            let _ = socket.read(&mut buf).await;
            tokio::time::sleep(Duration::from_secs(3600)).await;
        });
        let client = ZwipeClient::new(Url::parse(&format!("http://{addr}")).unwrap());
        let mut filter =
            zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder::default(
            );
        filter.set_name_contains("elf");
        let query = filter.build().unwrap();
        let session: zwipe_core::domain::auth::models::session::Session = serde_json::from_value(serde_json::json!({
                "user": {
                    "id": "00000000-0000-0000-0000-000000000001",
                    "username": "scotty",
                    "email": "scotty@zwipe.net",
                    "email_verified_at": null,
                },
                "access_token": {"value": "aGVhZA.cGF5bG9hZA.c2ln", "expires_at": "2030-01-01T01:00:00Z"},
                "refresh_token": {"value": "r", "expires_at": "2030-01-15T00:00:00Z"},
                "settings": zwipe_core::domain::user::UserSettings::default(),
            }))
            .unwrap();

        let started = tokio::time::Instant::now();
        let result = client
            .search_deck_cards(uuid::Uuid::nil(), &query, &session)
            .await;

        assert!(
            matches!(result, Err(ClientError::Network(_))),
            "a stalled search is a network error: {result:?}"
        );
        assert!(
            started.elapsed() <= Duration::from_secs(21),
            "gave up after {:?}, past the search's 20 seconds",
            started.elapsed()
        );
    }
}
