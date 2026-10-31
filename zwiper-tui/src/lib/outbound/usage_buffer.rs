//! Handled API errors waiting for the next usage batch, after the app's `UsageBuffer::report_error`.
//!
//! Each error is kept once per screen, component, action, kind and message, with a count, and at most [`HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH`] distinct errors wait at once: an error loop raises counts but never grows the buffer. The TUI has no swipe-count or signal telemetry, so a batch carries those at zero and empty.

use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex};

use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::http::contracts::metrics::{ClientErrorReport, HttpUsageBatch};

/// One error's dedupe key within a flush window.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ErrorKey {
    screen: &'static str,
    component: &'static str,
    action: &'static str,
    kind: &'static str,
    message: String,
}

/// Errors shown to the user since the last flush. Cloning shares the buffer.
#[derive(Debug, Clone, Default)]
pub(crate) struct UsageBuffer {
    errors: Arc<Mutex<HashMap<ErrorKey, u32>>>,
}

impl UsageBuffer {
    /// Keeps a handled error that was just shown to the user.
    ///
    /// `component` is `""` when the screen itself shows the error. Network errors are never kept: they are mostly the user's connection, and could not be sent over it anyway. A decode error keeps only its shape, with quoted input removed, so no response body rides a report.
    pub(crate) fn report_error(
        &self,
        screen: &'static str,
        component: &'static str,
        action: &'static str,
        error: &ClientError,
    ) {
        let (kind, message) = match error {
            ClientError::Network(_) => return,
            ClientError::Decode(detail) => {
                ("decode", sanitized_decode_message(&detail.to_string()))
            }
            api => (api_error_kind(api), api.to_string()),
        };
        let message: String = message
            .chars()
            .take(ClientErrorReport::MAX_MESSAGE_CHARS)
            .collect();
        let Ok(mut map) = self.errors.lock() else {
            return;
        };
        let key = ErrorKey {
            screen,
            component,
            action,
            kind,
            message,
        };
        if let Some(count) = map.get_mut(&key) {
            *count = count.saturating_add(1);
        } else if map.len() < HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH {
            map.insert(key, 1);
        }
    }

    /// Drops everything waiting without sending it, as when the session ends.
    pub(crate) fn clear(&self) {
        drop(self.snapshot_and_zero());
    }

    /// Takes everything waiting as a batch tagged with this build's version and the terminal platform, or `None` when nothing waits.
    pub(crate) fn snapshot_and_zero(&self) -> Option<HttpUsageBatch> {
        let errors = self
            .errors
            .lock()
            .map(|mut map| mem::take(&mut *map))
            .unwrap_or_default();
        if errors.is_empty() {
            return None;
        }
        let mut client_errors: Vec<ClientErrorReport> = errors
            .into_iter()
            .map(|(key, count)| ClientErrorReport {
                screen: key.screen.to_owned(),
                component: key.component.to_owned(),
                action: key.action.to_owned(),
                kind: key.kind.to_owned(),
                message: key.message,
                count,
                client_version: env!("CARGO_PKG_VERSION").to_owned(),
                platform: ClientPlatform::Terminal.as_str().to_owned(),
            })
            .collect();
        client_errors.sort_by(|a, b| (&a.screen, &a.action).cmp(&(&b.screen, &b.action)));
        Some(HttpUsageBatch {
            swipes_right: 0,
            swipes_left: 0,
            swipes_up: 0,
            swipes_down: 0,
            searches: 0,
            deck_skips: Vec::new(),
            signals: Vec::new(),
            select_signals: Vec::new(),
            client_errors,
            pile_ends: Vec::new(),
        })
    }
}

/// The stored kind slug for a status-carrying [`ClientError`], the app's fixed strings.
fn api_error_kind(api: &ClientError) -> &'static str {
    match api {
        ClientError::Unauthorized(_) => "api_unauthorized",
        ClientError::Forbidden(_) => "api_forbidden",
        ClientError::NotFound(_) => "api_not_found",
        ClientError::UnprocessableEntity(_) => "api_unprocessable",
        ClientError::TooManyRequests(_) => "api_too_many_requests",
        ClientError::Conflict(_) => "api_conflict",
        ClientError::BadRequest(_) => "api_bad_request",
        ClientError::PayloadTooLarge(_) => "api_payload_too_large",
        // Network and Decode are handled before this is reached.
        ClientError::InternalServerError(_) | ClientError::Network(_) | ClientError::Decode(_) => {
            "api_internal"
        }
    }
}

/// A decode error's message reduced to its shape: each double-quoted span becomes `"…"`, while field names in backticks and line and column positions stay.
fn sanitized_decode_message(detail: &str) -> String {
    let mut out = String::with_capacity(detail.len().min(128));
    let mut in_quotes = false;
    for c in detail.chars() {
        if c == '"' {
            if in_quotes {
                out.push('"');
            } else {
                out.push_str("\"…");
            }
            in_quotes = !in_quotes;
        } else if !in_quotes {
            out.push(c);
        }
    }
    if in_quotes {
        out.push('"');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_dedupe_and_cap() {
        let buffer = UsageBuffer::default();
        let error = ClientError::UnprocessableEntity("deck limit reached".to_owned());
        for _ in 0..3 {
            buffer.report_error("deck_form", "", "save", &error);
        }
        for i in 0..(HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH + 10) {
            let action: &'static str = Box::leak(format!("action_{i}").into_boxed_str());
            buffer.report_error("deck_form", "", action, &error);
        }

        let batch = buffer.snapshot_and_zero().expect("errors buffered");
        assert_eq!(
            batch.client_errors.len(),
            HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH,
            "distinct entries capped"
        );
        let repeated = batch
            .client_errors
            .iter()
            .find(|r| r.action == "save")
            .expect("deduped entry present");
        assert_eq!(repeated.count, 3);
        assert_eq!(repeated.kind, "api_unprocessable");
        assert_eq!(repeated.message, "deck limit reached");
        assert!(buffer.snapshot_and_zero().is_none(), "drained");
    }

    #[test]
    fn a_batch_carries_terminal_and_the_version_with_zero_counts() {
        let buffer = UsageBuffer::default();
        buffer.report_error(
            "decks",
            "",
            "load",
            &ClientError::InternalServerError("boom".to_owned()),
        );
        let batch = buffer.snapshot_and_zero().unwrap();
        let json = serde_json::to_string(&batch).unwrap();
        assert!(json.contains(r#""platform":"terminal""#), "{json}");
        let version = format!(r#""client_version":"{}""#, env!("CARGO_PKG_VERSION"));
        assert!(json.contains(&version), "{json}");
        assert_eq!(
            (
                batch.swipes_right,
                batch.swipes_left,
                batch.swipes_up,
                batch.swipes_down,
                batch.searches
            ),
            (0, 0, 0, 0, 0)
        );
    }

    #[test]
    fn network_errors_are_never_kept() {
        let buffer = UsageBuffer::default();
        let network = ClientError::from(
            reqwest::Client::new()
                .get("not a url")
                .build()
                .expect_err("a relative url fails to build"),
        );
        buffer.report_error("decks", "", "load", &network);
        assert!(buffer.snapshot_and_zero().is_none());
    }

    #[test]
    fn decode_messages_lose_quoted_input() {
        let buffer = UsageBuffer::default();
        let decode = ClientError::from(
            serde_json::from_str::<u32>("\"secret payload contents\"").expect_err("not a u32"),
        );
        buffer.report_error("deck", "", "load", &decode);
        let batch = buffer.snapshot_and_zero().unwrap();
        let report = batch.client_errors.first().unwrap();
        assert!(!report.message.contains("secret"), "{}", report.message);
        assert!(
            report.message.contains("expected u32 at line 1 column 25"),
            "{}",
            report.message
        );
    }
}
