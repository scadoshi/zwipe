//! A one-request server and a session for the collection calls' tests.

use reqwest::Url;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use zwipe_core::domain::auth::models::session::Session;

use crate::ZwipeClient;

/// A server that answers one request with `status` and the JSON `body`, handing back the request
/// it read, and a client pointed at it.
#[expect(
    clippy::unwrap_used,
    reason = "a test helper: a socket error fails the test"
)]
pub(super) async fn serve_once(status: &str, body: String) -> (ZwipeClient, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let reply = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = vec![0_u8; 16384];
        let read = socket.read(&mut buf).await.unwrap();
        let request = String::from_utf8_lossy(buf.get(..read).unwrap()).into_owned();
        socket.write_all(reply.as_bytes()).await.unwrap();
        request
    });
    let client = ZwipeClient::new(Url::parse(&format!("http://{addr}")).unwrap());
    (client, server)
}

/// A signed-in session.
#[expect(
    clippy::unwrap_used,
    reason = "a test helper: a fixed session always parses"
)]
pub(super) fn session() -> Session {
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

/// Whether `request` carries the [`session`]'s token.
pub(super) fn has_token(request: &str) -> bool {
    request
        .to_ascii_lowercase()
        .contains("authorization: bearer agvhza.cgf5bg9hza.c2ln")
}
