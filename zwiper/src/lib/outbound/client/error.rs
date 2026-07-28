//! Client-side error type: the app's error currency.
//!
//! Owns the two translations the server can't see:
//! - transport/decode failures ([`ClientError::Network`] / [`ClientError::Decode`]) —
//!   these never crossed the wire, so they don't belong in the shared [`ApiError`]
//!   vocabulary;
//! - wire responses back into vocabulary via `From<(StatusCode, String)>`.
//!
//! User-facing copy lives here too ([`ClientError::to_user_message`]) — the
//! client owns the client-to-user translation. Server-authored 4xx messages
//! pass through verbatim: by contract they are user-safe copy.

use reqwest::StatusCode;
use thiserror::Error;
use zwipe::inbound::http::ApiError;

/// Errors surfaced by API client calls.
#[derive(Debug, Error, Clone)]
pub enum ClientError {
    /// The server responded with an error status ([`ApiError`] vocabulary).
    #[error("{0}")]
    Api(ApiError),
    /// The request never completed: connection, TLS, timeout, DNS.
    #[error("network error: {0}")]
    Network(String),
    /// The response arrived but its body couldn't be decoded.
    #[error("decode error: {0}")]
    Decode(String),
}

impl ClientError {
    /// Returns a safe, user-facing message — never leaks internal details like URLs or stack traces.
    pub fn to_user_message(&self) -> String {
        match self {
            ClientError::Network(_) => {
                "connection error, check your network and try again".to_string()
            }
            ClientError::Decode(_) | ClientError::Api(ApiError::InternalServerError(_)) => {
                "something went wrong, please try again".to_string()
            }
            ClientError::Api(other) => other.to_string(),
        }
    }
}

impl From<ApiError> for ClientError {
    fn from(value: ApiError) -> Self {
        Self::Api(value)
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
        Self::Api(match status {
            StatusCode::UNAUTHORIZED => ApiError::Unauthorized(message),
            StatusCode::FORBIDDEN => ApiError::Forbidden(message),
            StatusCode::NOT_FOUND => ApiError::NotFound(message),
            StatusCode::UNPROCESSABLE_ENTITY => ApiError::UnprocessableEntity(message),
            StatusCode::TOO_MANY_REQUESTS => ApiError::TooManyRequests(message),
            _ => ApiError::InternalServerError(message),
        })
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
            let ClientError::Api(api) = &error else {
                panic!("expected Api variant for {status}");
            };
            let actual = match api {
                ApiError::Unauthorized(_) => "unauthorized",
                ApiError::Forbidden(_) => "forbidden",
                ApiError::NotFound(_) => "not_found",
                ApiError::UnprocessableEntity(_) => "unprocessable",
                ApiError::TooManyRequests(_) => "too_many",
                ApiError::InternalServerError(_) => "internal",
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
        let internal = ClientError::Api(ApiError::InternalServerError("pg pool timeout".into()));
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

        let four_xx = ClientError::Api(ApiError::UnprocessableEntity(
            "deck name cannot be empty".into(),
        ));
        assert_eq!(four_xx.to_user_message(), "deck name cannot be empty");
    }
}
