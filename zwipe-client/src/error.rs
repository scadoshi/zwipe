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
//! client owns the client-to-user translation. A 4xx the user caused and can
//! fix (a bad import link, a taken name, a full deck) is matched against the
//! server's text in [`zwipe_core::http::messages`] and shown in plain words;
//! any other server-authored 4xx passes through verbatim, since by contract
//! it is user-safe copy. A 400 or 413
//! comes from the framework rather than the server's `ApiError`, so its body
//! is replaced with copy of the client's own.

use std::sync::Arc;

use reqwest::StatusCode;
use thiserror::Error;
use zwipe_core::domain::deck::requests::import_deck_cards::{MAX_IMPORT_LINES, TooManyImportLines};
use zwipe_core::http::messages;
use zwipe_core::text::sentence_case;

/// Errors surfaced by API client calls.
#[derive(Debug, Error, Clone)]
pub enum ClientError {
    /// 400: the server could not read the request at all.
    #[error("{0}")]
    BadRequest(String),
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
    /// 409: the request clashes with the current state.
    #[error("{0}")]
    Conflict(String),
    /// 413: the request body is over the server's size limit.
    #[error("{0}")]
    PayloadTooLarge(String),
    /// 429: rate limited.
    #[error("{0}")]
    TooManyRequests(String),
    /// 5xx, and any status without a variant of its own.
    #[error("{0}")]
    InternalServerError(String),
    /// The request never completed (connection, TLS, timeout, DNS), or its
    /// body couldn't be read off the wire. `Arc` keeps the error `Clone`.
    #[error("network error: {0}")]
    Network(#[source] Arc<reqwest::Error>),
    /// The response arrived but its body isn't the JSON the endpoint expects.
    #[error("decode error: {0}")]
    Decode(#[source] Arc<serde_json::Error>),
}

impl ClientError {
    /// The HTTP status of the server answer this error came from, or `None`
    /// when no status reached the client: a [`ClientError::Network`] failure
    /// or a [`ClientError::Decode`] of a body that arrived unreadable.
    ///
    /// [`ClientError::InternalServerError`] reports 500 for every status it
    /// stands for, since it keeps only the message: a 502 or 503 reads as 500,
    /// and so does one a client builds for a failure of its own.
    #[must_use]
    pub fn status(&self) -> Option<u16> {
        match self {
            ClientError::BadRequest(_) => Some(400),
            ClientError::Unauthorized(_) => Some(401),
            ClientError::Forbidden(_) => Some(403),
            ClientError::NotFound(_) => Some(404),
            ClientError::Conflict(_) => Some(409),
            ClientError::PayloadTooLarge(_) => Some(413),
            ClientError::UnprocessableEntity(_) => Some(422),
            ClientError::TooManyRequests(_) => Some(429),
            ClientError::InternalServerError(_) => Some(500),
            ClientError::Network(_) | ClientError::Decode(_) => None,
        }
    }

    /// Returns a safe, user-facing message in sentence case: never leaks internal details like URLs or stack traces. The server's own lowercase message gets its first letter capitalized.
    pub fn to_user_message(&self) -> String {
        let message = match self {
            ClientError::Network(_) => {
                "Connection error, check your network and try again".to_string()
            }
            ClientError::BadRequest(_)
            | ClientError::Decode(_)
            | ClientError::InternalServerError(_) => SOMETHING_WENT_WRONG.to_string(),
            ClientError::PayloadTooLarge(_) => {
                "Request too large, try importing fewer cards".to_string()
            }
            ClientError::NotFound(message)
            | ClientError::Conflict(message)
            | ClientError::UnprocessableEntity(message) => {
                plain_message(message).unwrap_or_else(|| sentence_case(message))
            }
            other => sentence_case(&other.to_string()),
        };
        // A status with an empty body (a route the server doesn't have
        // yet answers 404 with none) would otherwise toast nothing.
        if message.trim().is_empty() {
            SOMETHING_WENT_WRONG.to_string()
        } else {
            message
        }
    }
}

/// What a failure says when nothing more specific can be said.
const SOMETHING_WENT_WRONG: &str = "Something went wrong, please try again";

/// Plain words for a server message about a mistake the user can fix, or
/// `None` for any other message, which shows as the server wrote it.
fn plain_message(server: &str) -> Option<String> {
    let plain = match server {
        messages::ARCHIDEKT_BAD_LINK => "That isn't an Archidekt deck link",
        messages::ARCHIDEKT_NOT_FOUND => "That Archidekt deck is private or doesn't exist",
        messages::ARCHIDEKT_TOO_LARGE => "That Archidekt deck is too big to import",
        messages::CARD_LIMIT => "This deck is full (every board counts toward the limit)",
        messages::UNVERIFIED_CARD_LIMIT => "Verify your email to add more cards to this deck",
        messages::DECK_NAME_TAKEN | messages::CLONE_NAME_TAKEN => {
            "You already have a deck with that name"
        }
        messages::USERNAME_TAKEN | messages::USERNAME_IN_USE => "That username is taken",
        messages::EMAIL_IN_USE => "Another account already uses that email",
        messages::CARD_ALREADY_IN_DECK => "That card is already in this deck",
        messages::PASSWORD_TOO_LONG => "That password is too long",
        _ if server == TooManyImportLines.to_string() => {
            return Some(format!(
                "Paste up to {MAX_IMPORT_LINES} card lines at a time"
            ));
        }
        _ => return None,
    };
    Some(plain.to_owned())
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(Arc::new(value))
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(value: serde_json::Error) -> Self {
        Self::Decode(Arc::new(value))
    }
}

impl From<(StatusCode, String)> for ClientError {
    fn from(value: (StatusCode, String)) -> Self {
        let (status, message) = value;
        match status {
            StatusCode::BAD_REQUEST => Self::BadRequest(message),
            StatusCode::UNAUTHORIZED => Self::Unauthorized(message),
            StatusCode::FORBIDDEN => Self::Forbidden(message),
            StatusCode::NOT_FOUND => Self::NotFound(message),
            StatusCode::UNPROCESSABLE_ENTITY => Self::UnprocessableEntity(message),
            StatusCode::TOO_MANY_REQUESTS => Self::TooManyRequests(message),
            StatusCode::CONFLICT => Self::Conflict(message),
            StatusCode::PAYLOAD_TOO_LARGE => Self::PayloadTooLarge(message),
            _ => Self::InternalServerError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failure_with_no_message_still_says_something() {
        // A route the server doesn't have yet answers 404 with an empty body.
        for message in ["", "   ", "\n"] {
            let errors = [
                ClientError::Unauthorized(message.to_owned()),
                ClientError::Forbidden(message.to_owned()),
                ClientError::NotFound(message.to_owned()),
                ClientError::Conflict(message.to_owned()),
                ClientError::UnprocessableEntity(message.to_owned()),
                ClientError::TooManyRequests(message.to_owned()),
                ClientError::InternalServerError(message.to_owned()),
                ClientError::BadRequest(message.to_owned()),
                ClientError::PayloadTooLarge(message.to_owned()),
            ];
            for error in errors {
                let shown = error.to_user_message();
                assert!(!shown.trim().is_empty(), "{error:?} shows nothing");
            }
        }
        assert_eq!(
            ClientError::NotFound(String::new()).to_user_message(),
            "Something went wrong, please try again"
        );
        assert_eq!(
            ClientError::NotFound("deck not found".to_owned()).to_user_message(),
            "Deck not found",
            "a message the server sent still shows"
        );
    }

    #[test]
    fn a_mistake_the_user_can_fix_reads_plainly() {
        let cases = [
            (
                ClientError::UnprocessableEntity(messages::ARCHIDEKT_BAD_LINK.to_owned()),
                "That isn't an Archidekt deck link",
            ),
            (
                ClientError::NotFound(messages::ARCHIDEKT_NOT_FOUND.to_owned()),
                "That Archidekt deck is private or doesn't exist",
            ),
            (
                ClientError::UnprocessableEntity(messages::ARCHIDEKT_TOO_LARGE.to_owned()),
                "That Archidekt deck is too big to import",
            ),
            (
                ClientError::UnprocessableEntity(TooManyImportLines.to_string()),
                "Paste up to 1000 card lines at a time",
            ),
            (
                ClientError::UnprocessableEntity(messages::CARD_LIMIT.to_owned()),
                "This deck is full (every board counts toward the limit)",
            ),
            (
                ClientError::UnprocessableEntity(messages::UNVERIFIED_CARD_LIMIT.to_owned()),
                "Verify your email to add more cards to this deck",
            ),
            (
                ClientError::Conflict(messages::DECK_NAME_TAKEN.to_owned()),
                "You already have a deck with that name",
            ),
            (
                ClientError::Conflict(messages::CLONE_NAME_TAKEN.to_owned()),
                "You already have a deck with that name",
            ),
            (
                ClientError::Conflict(messages::USERNAME_TAKEN.to_owned()),
                "That username is taken",
            ),
            (
                ClientError::Conflict(messages::USERNAME_IN_USE.to_owned()),
                "That username is taken",
            ),
            (
                ClientError::Conflict(messages::EMAIL_IN_USE.to_owned()),
                "Another account already uses that email",
            ),
            (
                ClientError::Conflict(messages::CARD_ALREADY_IN_DECK.to_owned()),
                "That card is already in this deck",
            ),
            (
                ClientError::UnprocessableEntity(messages::PASSWORD_TOO_LONG.to_owned()),
                "That password is too long",
            ),
        ];
        for (error, plain) in cases {
            assert_eq!(error.to_user_message(), plain, "{error:?}");
        }
    }

    #[test]
    fn other_server_messages_and_real_failures_stay_as_they_were() {
        assert_eq!(
            ClientError::NotFound("deck not found".to_owned()).to_user_message(),
            "Deck not found"
        );
        assert_eq!(
            ClientError::UnprocessableEntity("invalid board value".to_owned()).to_user_message(),
            "Invalid board value"
        );
        assert_eq!(
            ClientError::InternalServerError(messages::ARCHIDEKT_BAD_LINK.to_owned())
                .to_user_message(),
            "Something went wrong, please try again",
            "only a 4xx is matched"
        );
        assert_eq!(
            ClientError::Unauthorized("invalid credentials".to_owned()).to_user_message(),
            "Invalid credentials"
        );
    }

    #[test]
    fn status_codes_map_to_vocabulary() {
        let cases = [
            (StatusCode::UNAUTHORIZED, "unauthorized"),
            (StatusCode::FORBIDDEN, "forbidden"),
            (StatusCode::NOT_FOUND, "not_found"),
            (StatusCode::UNPROCESSABLE_ENTITY, "unprocessable"),
            (StatusCode::TOO_MANY_REQUESTS, "too_many"),
            (StatusCode::CONFLICT, "conflict"),
            (StatusCode::BAD_REQUEST, "bad_request"),
            (StatusCode::PAYLOAD_TOO_LARGE, "payload_too_large"),
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
                ClientError::Conflict(_) => "conflict",
                ClientError::BadRequest(_) => "bad_request",
                ClientError::PayloadTooLarge(_) => "payload_too_large",
                ClientError::InternalServerError(_) => "internal",
                other => panic!("expected a status variant for {status}, got {other:?}"),
            };
            assert_eq!(actual, expected, "wrong variant for {status}");
        }
    }

    /// Transport and decode failures keep the underlying error as their
    /// source, so a log or report can walk the chain.
    #[test]
    fn network_and_decode_errors_keep_their_source() {
        let decode = ClientError::from(serde_json::from_str::<u32>("\"x\"").unwrap_err());
        assert!(matches!(decode, ClientError::Decode(_)));
        assert!(std::error::Error::source(&decode).is_some());

        let builder = reqwest::Client::new().get("not a url").build().unwrap_err();
        let network = ClientError::from(builder);
        assert!(matches!(network, ClientError::Network(_)));
        assert!(std::error::Error::source(&network).is_some());
    }

    #[test]
    fn oversized_requests_say_so() {
        let error = ClientError::from((
            StatusCode::PAYLOAD_TOO_LARGE,
            "length limit exceeded".to_string(),
        ));
        assert_eq!(
            error.to_user_message(),
            "Request too large, try importing fewer cards"
        );
    }

    #[test]
    fn wire_messages_pass_through_in_sentence_case() {
        let error = ClientError::from((StatusCode::NOT_FOUND, "Deck Not Found".to_string()));
        assert_eq!(error.to_user_message(), "Deck Not Found");
        let error = ClientError::from((StatusCode::CONFLICT, "deck limit reached".to_string()));
        assert_eq!(error.to_user_message(), "Deck limit reached");
    }

    /// Internal detail, transport noise, and decode noise all collapse to
    /// generic copy; user-safe 4xx messages pass through.
    #[test]
    fn user_messages_never_expose_internals() {
        let internal = ClientError::InternalServerError("pg pool timeout".into());
        assert_eq!(
            internal.to_user_message(),
            "Something went wrong, please try again"
        );

        let network = ClientError::from(
            reqwest::Client::new()
                .get("not a url: api.zwipe.net")
                .build()
                .unwrap_err(),
        );
        assert_eq!(
            network.to_user_message(),
            "Connection error, check your network and try again"
        );

        let decode =
            ClientError::from(serde_json::from_str::<u32>("\"missing field\"").unwrap_err());
        assert_eq!(
            decode.to_user_message(),
            "Something went wrong, please try again"
        );

        let bad_request = ClientError::BadRequest(
            "failed to parse the request body as json: expected value at line 1".into(),
        );
        assert_eq!(
            bad_request.to_user_message(),
            "Something went wrong, please try again"
        );

        let four_xx = ClientError::UnprocessableEntity("deck name cannot be empty".into());
        assert_eq!(four_xx.to_user_message(), "Deck name cannot be empty");
    }

    /// Every status variant reports the status it was built from, so a
    /// status survives the round trip through the vocabulary.
    #[test]
    fn status_variants_report_their_status() {
        let statuses = [
            StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::NOT_FOUND,
            StatusCode::CONFLICT,
            StatusCode::PAYLOAD_TOO_LARGE,
            StatusCode::UNPROCESSABLE_ENTITY,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::INTERNAL_SERVER_ERROR,
        ];
        for status in statuses {
            let error = ClientError::from((status, "Message".to_string()));
            assert_eq!(error.status(), Some(status.as_u16()), "{error:?}");
        }
    }

    #[test]
    fn statuses_without_a_variant_report_500() {
        for status in [StatusCode::BAD_GATEWAY, StatusCode::IM_A_TEAPOT] {
            let error = ClientError::from((status, "Message".to_string()));
            assert_eq!(error.status(), Some(500), "{status}");
        }
    }

    #[test]
    fn network_and_decode_errors_have_no_status() {
        let decode = ClientError::from(serde_json::from_str::<u32>("\"x\"").unwrap_err());
        assert_eq!(decode.status(), None);

        let builder = reqwest::Client::new().get("not a url").build().unwrap_err();
        assert_eq!(ClientError::from(builder).status(), None);
    }
}
