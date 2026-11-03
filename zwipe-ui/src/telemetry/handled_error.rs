//! A handled error a client showed the user, as it rides a usage batch.

use zwipe_core::http::contracts::metrics::ClientErrorReport;

/// What kind of failure a handled error was. The slugs from [`as_str`](Self::as_str) are stored server side in `client_errors.kind`, so they never change.
///
/// A network failure has no kind: it is mostly the user's connection, and could not be reported over it anyway, so clients log it and never buffer it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorKind {
    /// The response arrived but did not parse.
    Decode,
    /// 401.
    Unauthorized,
    /// 403.
    Forbidden,
    /// 404.
    NotFound,
    /// 422.
    Unprocessable,
    /// 429.
    TooManyRequests,
    /// 409.
    Conflict,
    /// 400.
    BadRequest,
    /// 413.
    PayloadTooLarge,
    /// 500, and any status a client does not name.
    Internal,
}

impl ErrorKind {
    /// The stored slug for this kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Decode => "decode",
            Self::Unauthorized => "api_unauthorized",
            Self::Forbidden => "api_forbidden",
            Self::NotFound => "api_not_found",
            Self::Unprocessable => "api_unprocessable",
            Self::TooManyRequests => "api_too_many_requests",
            Self::Conflict => "api_conflict",
            Self::BadRequest => "api_bad_request",
            Self::PayloadTooLarge => "api_payload_too_large",
            Self::Internal => "api_internal",
        }
    }
}

/// One handled error, ready to log and to buffer. Two equal values are the same error, so the buffer counts repeats of one instead of keeping each.
///
/// `screen` and `component` come from [`vocabulary`](super::vocabulary); `component` is [`component::NONE`](super::vocabulary::component::NONE) when the screen itself showed the error. `action` names what the user was doing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HandledError {
    screen: &'static str,
    component: &'static str,
    action: &'static str,
    kind: ErrorKind,
    message: String,
}

impl HandledError {
    /// Builds the report from the error's display text.
    ///
    /// A [`ErrorKind::Decode`] message keeps only its shape: each double-quoted span becomes `"…"`, so no response body rides a report or a log line, while field names in backticks and line and column positions stay. Every message is then cut to [`ClientErrorReport::MAX_MESSAGE_CHARS`] characters.
    #[must_use]
    pub fn new(
        screen: &'static str,
        component: &'static str,
        action: &'static str,
        kind: ErrorKind,
        message: &str,
    ) -> Self {
        let shaped = match kind {
            ErrorKind::Decode => decode_shape(message),
            _ => message.to_owned(),
        };
        let message: String = shaped
            .chars()
            .take(ClientErrorReport::MAX_MESSAGE_CHARS)
            .collect();
        Self {
            screen,
            component,
            action,
            kind,
            message,
        }
    }

    /// The screen that showed the error.
    #[must_use]
    pub const fn screen(&self) -> &'static str {
        self.screen
    }

    /// The component that showed it, or `""` for the screen itself.
    #[must_use]
    pub const fn component(&self) -> &'static str {
        self.component
    }

    /// What the user was doing.
    #[must_use]
    pub const fn action(&self) -> &'static str {
        self.action
    }

    /// The kind of failure.
    #[must_use]
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The message as it is reported: shaped and cut.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The wire report for `count` repeats, tagged with the sending build.
    pub(super) fn into_report(
        self,
        count: u32,
        client_version: &str,
        platform: &str,
    ) -> ClientErrorReport {
        ClientErrorReport {
            screen: self.screen.to_owned(),
            component: self.component.to_owned(),
            action: self.action.to_owned(),
            kind: self.kind.as_str().to_owned(),
            message: self.message,
            count,
            client_version: client_version.to_owned(),
            platform: platform.to_owned(),
        }
    }
}

/// A decode error's message reduced to its shape: serde quotes fragments of the input around the failure (`invalid type: string "..."`), so each double-quoted span collapses to `"…"`. An unclosed quote is closed.
fn decode_shape(detail: &str) -> String {
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
    fn the_slugs_are_the_stored_strings() {
        let slugs = [
            (ErrorKind::Decode, "decode"),
            (ErrorKind::Unauthorized, "api_unauthorized"),
            (ErrorKind::Forbidden, "api_forbidden"),
            (ErrorKind::NotFound, "api_not_found"),
            (ErrorKind::Unprocessable, "api_unprocessable"),
            (ErrorKind::TooManyRequests, "api_too_many_requests"),
            (ErrorKind::Conflict, "api_conflict"),
            (ErrorKind::BadRequest, "api_bad_request"),
            (ErrorKind::PayloadTooLarge, "api_payload_too_large"),
            (ErrorKind::Internal, "api_internal"),
        ];
        for (kind, slug) in slugs {
            assert_eq!(kind.as_str(), slug, "{kind:?}");
        }
    }

    #[test]
    fn decode_messages_lose_quoted_input_fragments() {
        let detail = serde_json::from_str::<u32>("\"secret payload contents\"")
            .expect_err("a string is not a u32")
            .to_string();
        let error = HandledError::new("deck_view", "", "load_deck", ErrorKind::Decode, &detail);
        assert!(
            !error.message().contains("secret payload contents"),
            "quoted input stripped: {}",
            error.message()
        );
        assert!(
            error.message().contains("expected u32 at line 1 column 25"),
            "error shape survives: {}",
            error.message()
        );
    }

    #[test]
    fn an_unclosed_quote_is_closed() {
        assert_eq!(decode_shape(r#"bad "tail"#), r#"bad "…""#);
    }

    #[test]
    fn only_decode_messages_are_shaped() {
        let error = HandledError::new("deck", "", "save", ErrorKind::Conflict, r#"name "x" taken"#);
        assert_eq!(error.message(), r#"name "x" taken"#);
    }

    #[test]
    fn messages_are_cut_to_the_report_limit() {
        let long = "é".repeat(ClientErrorReport::MAX_MESSAGE_CHARS + 5);
        let error = HandledError::new("deck", "", "save", ErrorKind::Internal, &long);
        assert_eq!(
            error.message().chars().count(),
            ClientErrorReport::MAX_MESSAGE_CHARS
        );
    }
}
