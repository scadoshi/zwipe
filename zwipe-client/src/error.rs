//! Client-side error type: the app's error currency.
//!
//! The status vocabulary is the client's own copy of the protocol, built from
//! `(status, body)` in `From<(StatusCode, String)>`. The server names the same
//! statuses in its `ApiError`; neither type crosses the wire, so each side
//! keeps its own.
//!
//! Transport and decode failures ([`ClientError::Network`] /
//! [`ClientError::Decode`]) never reached the server at all.
//!
//! User-facing copy lives here too ([`ClientError::to_user_message`]), the
//! client owns the client-to-user translation. Server-authored 4xx messages
//! pass through verbatim: by contract they are user-safe copy.

use reqwest::StatusCode;
use thiserror::Error;

/// Errors surfaced by API client calls.
#[derive(Debug, Error, Clone)]
pub enum ClientError {
    /// 401: no valid session.
    #[error("{0}")]
    Unauthorized(String),
    /// 403: authenticated, but not allowed.
    #[error("{0}")]
    Forbidden(String),
    /// 404: no such resource.
    #[error("{0}")]
    NotFound(String),
    /// 422: the request was understood but rejected.
    #[error("{0}")]
    UnprocessableEntity(String),
    /// 429: rate limited.
    #[error("{0}")]
    TooManyRequests(String),
    /// 5xx, and any status without a variant of its own.
    #[error("{0}")]
    InternalServerError(String),
    /// The request never completed: connection, TLS, timeout, DNS.
    #[error("network error: {0}")]
    Network(String),
    /// The response arrived but its body couldn't be decoded.
    #[error("decode error: {0}")]
    Decode(String),
}

impl ClientError {
    /// Returns a safe, user-facing message: never leaks internal details like URLs or stack traces.
    pub fn to_user_message(&self) -> String {
        match self {
            ClientError::Network(_) => {
                "connection error, check your network and try again".to_string()
            }
            ClientError::Decode(_) | ClientError::InternalServerError(_) => {
                "something went wrong, please try again".to_string()
            }
            other => other.to_string(),
        }
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_decode() {
            Self::Decode(value.to_string())
        } else {
            Self::Network(value.to_string())
        }
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(value: serde_json::Error) -> Self {
        Self::Decode(value.to_string())
    }
}

impl From<(StatusCode, String)> for ClientError {
    fn from(value: (StatusCode, String)) -> Self {
        let (status, message) = value;
        let message = message.to_lowercase();
        match status {
            StatusCode::UNAUTHORIZED => Self::Unauthorized(message),
            StatusCode::FORBIDDEN => Self::Forbidden(message),
            StatusCode::NOT_FOUND => Self::NotFound(message),
            StatusCode::UNPROCESSABLE_ENTITY => Self::UnprocessableEntity(message),
            StatusCode::TOO_MANY_REQUESTS => Self::TooManyRequests(message),
            _ => Self::InternalServerError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]
    use super::*;

    #[test]
    fn status_codes_map_to_vocabulary() {
        let cases = [
            (StatusCode::UNAUTHORIZED, "unauthorized"),
            (StatusCode::FORBIDDEN, "forbidden"),
            (StatusCode::NOT_FOUND, "not_found"),
            (StatusCode::UNPROCESSABLE_ENTITY, "unprocessable"),
            (StatusCode::TOO_MANY_REQUESTS, "too_many"),
            (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
            (StatusCode::BAD_GATEWAY, "internal"), // unknown statuses fold to 500
        ];
        for (status, expected) in cases {
            let error = ClientError::from((status, "Message".to_string()));
            let actual = match error {
                ClientError::Unauthorized(_) => "unauthorized",
                ClientError::Forbidden(_) => "forbidden",
                ClientError::NotFound(_) => "not_found",
                ClientError::UnprocessableEntity(_) => "unprocessable",
                ClientError::TooManyRequests(_) => "too_many",
                ClientError::InternalServerError(_) => "internal",
                other => panic!("expected a status variant for {status}, got {other:?}"),
            };
            assert_eq!(actual, expected, "wrong variant for {status}");
        }
    }

    #[test]
    fn wire_messages_are_lowercased() {
        let error = ClientError::from((StatusCode::NOT_FOUND, "Deck Not Found".to_string()));
        assert_eq!(error.to_user_message(), "deck not found");
    }

    /// Internal detail, transport noise, and decode noise all collapse to
    /// generic copy; user-safe 4xx messages pass through.
    #[test]
    fn user_messages_never_expose_internals() {
        let internal = ClientError::InternalServerError("pg pool timeout".into());
        assert_eq!(
            internal.to_user_message(),
            "something went wrong, please try again"
        );

        let network = ClientError::Network("dns error: no such host api.zwipe.net".into());
        assert_eq!(
            network.to_user_message(),
            "connection error, check your network and try again"
        );

        let decode = ClientError::Decode("missing field `id` at line 1".into());
        assert_eq!(
            decode.to_user_message(),
            "something went wrong, please try again"
        );

        let four_xx = ClientError::UnprocessableEntity("deck name cannot be empty".into());
        assert_eq!(four_xx.to_user_message(), "deck name cannot be empty");
    }
}
