//! A one-request HTTP server on localhost, for tests that need to see what the client sent.

use reqwest::Url;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// Serves one request with `status` (such as `"204 No Content"`) and `body`, and returns the base URL and a handle that yields the request as text.
pub(crate) async fn serve_once(
    status: &'static str,
    body: &'static str,
) -> (Url, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap())
        .parse()
        .unwrap();
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut seen = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = socket.read(&mut buf).await.unwrap();
            seen.extend_from_slice(&buf[..n]);
            let text = String::from_utf8_lossy(&seen).to_string();
            if let Some(end) = text.find("\r\n\r\n") {
                let length = text[..end]
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if seen.len() >= end.saturating_add(4).saturating_add(length) {
                    break;
                }
            }
            if n == 0 {
                break;
            }
        }
        let reply = format!(
            "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(reply.as_bytes()).await.unwrap();
        String::from_utf8_lossy(&seen).to_string()
    });
    (url, handle)
}
