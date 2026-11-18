//! Look cards up by name for the card search, signed in or out.

use std::time::Duration;

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::HttpCardLookup;
use zwipe_core::http::endpoints::card::LookupCards;

use crate::call::{decode, retry_after};
use crate::{ClientError, ZwipeClient};

/// Why a card lookup failed.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LookupError {
    /// 429: too many lookups from this address.
    #[error("too many lookups")]
    RateLimited {
        /// The response's `Retry-After`, or `None` when it carries none the client can read (a
        /// browser reads it only when the server exposes it).
        retry_after: Option<Duration>,
    },
    /// Anything else the call can fail with.
    #[error(transparent)]
    Client(#[from] ClientError),
}

impl ZwipeClient {
    /// Looks up the cards `lookup` names, at most the server's limit, one printing each. With a
    /// `session` it sends its token, so the user's Universes Beyond setting applies; without one
    /// it looks up signed out. A token the server refuses (401) asks once more signed out, so a
    /// token that expired since the caller freshened it still finds cards.
    ///
    /// # Errors
    ///
    /// [`LookupError::RateLimited`] on a 429, with its `Retry-After` when readable, and
    /// [`LookupError::Client`] when the request fails to complete, the server answers with
    /// another error status, or the reply doesn't decode.
    pub async fn lookup_cards(
        &self,
        lookup: &HttpCardLookup,
        session: Option<&Session>,
    ) -> Result<Vec<Card>, LookupError> {
        let mut response = self
            .send(&LookupCards(lookup.clone()), session, None)
            .await?;
        if session.is_some() && response.status() == reqwest::StatusCode::UNAUTHORIZED {
            response = self.send(&LookupCards(lookup.clone()), None, None).await?;
        }
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(LookupError::RateLimited {
                retry_after: retry_after(response.headers()),
            });
        }
        Ok(decode::<LookupCards>(response).await?)
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use reqwest::Url;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use zwipe_core::domain::card::models::lookup::LookupName;

    use super::*;

    /// A server that answers one request with `reply` and hands back the request it read.
    #[expect(
        clippy::unwrap_used,
        reason = "a test helper: a socket or fixture error fails the test"
    )]
    async fn serve_once(reply: &'static [u8]) -> (ZwipeClient, tokio::task::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 8192];
            let read = socket.read(&mut buf).await.unwrap();
            let request = String::from_utf8_lossy(buf.get(..read).unwrap()).into_owned();
            socket.write_all(reply).await.unwrap();
            request
        });
        let client = ZwipeClient::new(Url::parse(&format!("http://{addr}")).unwrap());
        (client, server)
    }

    const EMPTY: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n[]";

    #[expect(
        clippy::unwrap_used,
        reason = "a test helper: a socket or fixture error fails the test"
    )]
    fn bolt() -> HttpCardLookup {
        let name = LookupName::try_from("bolt".to_owned()).unwrap();
        HttpCardLookup::try_new(name, Vec::new(), Vec::new(), Vec::new()).unwrap()
    }

    #[expect(
        clippy::unwrap_used,
        reason = "a test helper: a socket or fixture error fails the test"
    )]
    fn session() -> Session {
        serde_json::from_value(serde_json::json!({
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
        .unwrap()
    }

    #[tokio::test]
    async fn signed_in_it_sends_the_token() {
        let (client, server) = serve_once(EMPTY).await;
        let cards = client
            .lookup_cards(&bolt(), Some(&session()))
            .await
            .unwrap();
        assert_eq!(cards, Vec::<Card>::new());
        let request = server.await.unwrap();
        assert!(request.starts_with("POST /api/card/lookup "), "{request}");
        assert!(
            request
                .to_ascii_lowercase()
                .contains("authorization: bearer agvhza.cgf5bg9hza.c2ln"),
            "{request}"
        );
        assert!(request.contains(r#""name":"bolt""#), "{request}");
    }

    #[tokio::test]
    async fn signed_out_it_sends_no_token_and_still_asks() {
        let (client, server) = serve_once(EMPTY).await;
        let cards = client.lookup_cards(&bolt(), None).await.unwrap();
        assert_eq!(cards, Vec::<Card>::new());
        let request = server.await.unwrap();
        assert!(request.starts_with("POST /api/card/lookup "), "{request}");
        assert!(
            !request.to_ascii_lowercase().contains("authorization:"),
            "{request}"
        );
    }

    #[tokio::test]
    async fn a_429_reads_its_retry_after() {
        let (client, _server) = serve_once(b"HTTP/1.1 429 Too Many Requests\r\nRetry-After: 3\r\nContent-Length: 4\r\nConnection: close\r\n\r\nslow").await;
        let error = client.lookup_cards(&bolt(), None).await.unwrap_err();
        assert!(
            matches!(error, LookupError::RateLimited { retry_after: Some(wait) } if wait == Duration::from_secs(3)),
            "{error:?}"
        );

        let (client, _server) = serve_once(
            b"HTTP/1.1 429 Too Many Requests\r\nContent-Length: 4\r\nConnection: close\r\n\r\nslow",
        )
        .await;
        let error = client.lookup_cards(&bolt(), None).await.unwrap_err();
        assert!(
            matches!(error, LookupError::RateLimited { retry_after: None }),
            "{error:?}"
        );
    }

    /// A server that answers each of its first `replies.len()` connections with the next reply,
    /// handing back the requests it read.
    #[expect(
        clippy::unwrap_used,
        reason = "a test helper: a socket error fails the test"
    )]
    async fn serve_each(
        replies: &'static [&'static [u8]],
    ) -> (ZwipeClient, tokio::task::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let mut requests = Vec::new();
            for reply in replies {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buf = vec![0_u8; 8192];
                let read = socket.read(&mut buf).await.unwrap();
                requests.push(String::from_utf8_lossy(buf.get(..read).unwrap()).into_owned());
                socket.write_all(reply).await.unwrap();
            }
            requests
        });
        let client = ZwipeClient::new(Url::parse(&format!("http://{addr}")).unwrap());
        (client, server)
    }

    const REFUSED: &[u8] =
        b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 7\r\nConnection: close\r\n\r\nexpired";

    /// A token the server refuses (a 401 for one that expired since the session was checked)
    /// costs the user nothing: the lookup asks once more signed out.
    #[tokio::test]
    async fn a_refused_token_asks_once_more_signed_out() {
        let (client, server) = serve_each(&[REFUSED, EMPTY]).await;
        let cards = client
            .lookup_cards(&bolt(), Some(&session()))
            .await
            .unwrap();
        assert_eq!(cards, Vec::<Card>::new());
        let requests = server.await.unwrap();
        assert!(
            requests[0]
                .to_ascii_lowercase()
                .contains("authorization: bearer")
        );
        assert!(!requests[1].to_ascii_lowercase().contains("authorization:"));
    }

    #[tokio::test]
    async fn a_401_signed_out_is_not_asked_again() {
        let (client, _server) = serve_once(REFUSED).await;
        let error = client.lookup_cards(&bolt(), None).await.unwrap_err();
        assert!(
            matches!(error, LookupError::Client(ClientError::Unauthorized(_))),
            "{error:?}"
        );
    }

    #[tokio::test]
    async fn another_error_is_the_client_s() {
        let (client, _server) = serve_once(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 4\r\nConnection: close\r\n\r\noops").await;
        let error = client.lookup_cards(&bolt(), None).await.unwrap_err();
        assert!(
            matches!(
                error,
                LookupError::Client(ClientError::InternalServerError(_))
            ),
            "{error:?}"
        );
    }
}
