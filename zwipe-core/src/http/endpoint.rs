//! One description per API call: method, path, auth, body and response type.
//!
//! An endpoint is a value, so paths that interpolate an id carry it
//! (`GetDeck(deck_id)`). Clients drive every call from this, which is what
//! keeps method and path from drifting apart: `/api/deck` is a POST to create
//! and a GET to list, and only this trait can say which.
//!
//! Transport lives in the client, not here; core stays free of reqwest.

use serde::{Serialize, de::DeserializeOwned};
use std::borrow::Cow;

/// HTTP methods the API uses. Defined here so core needs no `http` dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl Method {
    /// The wire spelling, e.g. `"GET"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
        }
    }
}

/// A single API call.
///
/// Both halves of the contract are typed. `Request` is the body this call
/// accepts and `Response` is what it answers, so a mismatched body is a
/// compile error rather than a 422 discovered on a device.
///
/// `Response` is `()` for endpoints that answer 204: an empty body decodes as
/// `null`, which is exactly what `()` deserializes from. `Request` is `()` for
/// the bodyless ones, which never send it. Associated-type defaults are
/// unstable, so those spell it out.
pub trait Endpoint {
    /// HTTP method.
    const METHOD: Method;
    /// Whether the call carries a bearer token.
    const AUTH: bool = true;
    /// Body this call accepts. `()` when it sends none.
    type Request: Serialize;
    /// Decoded success body.
    type Response: DeserializeOwned;

    /// Absolute path, ids interpolated.
    ///
    /// `Cow` so a fixed path borrows its const instead of allocating a copy
    /// of it; only the id-carrying paths build a `String`.
    fn path(&self) -> Cow<'static, str>;

    /// JSON request body, or `None` to send none.
    ///
    /// Bodyless calls must return `None`: sending `null` on a GET would put a
    /// body on the wire where shipped clients send none.
    fn body(&self) -> Option<&Self::Request> {
        None
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// The 204 contract: an empty body is read as `null`, and `()`
    /// deserializes from `null`. Endpoints returning no content depend on it.
    #[test]
    fn unit_decodes_from_null() {
        let decoded: () = serde_json::from_str("null").unwrap();
        assert_eq!(decoded, ());
    }

    /// Typing the request changed how bodies are serialized: they used to go
    /// through `serde_json::Value`, whose map is a `BTreeMap` here, so keys
    /// went out alphabetically. A typed `.json()` emits declaration order.
    ///
    /// That is a byte-level change to every mutating request, so this pins
    /// what actually matters: the two are the same document. JSON object
    /// order carries no meaning and serde ignores it when decoding.
    #[test]
    fn typing_the_body_changes_key_order_but_not_the_document() {
        #[derive(serde::Serialize)]
        struct Body {
            zeta: u8,
            alpha: u8,
        }
        let body = Body { zeta: 1, alpha: 2 };

        let typed = serde_json::to_string(&body).unwrap();
        let via_value = serde_json::to_string(&serde_json::to_value(&body).unwrap()).unwrap();

        assert_ne!(typed, via_value, "if these match, the premise changed");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&typed).unwrap(),
            serde_json::from_str::<serde_json::Value>(&via_value).unwrap(),
            "same document, different key order"
        );
    }

    #[test]
    fn methods_spell_themselves() {
        assert_eq!(Method::Get.as_str(), "GET");
        assert_eq!(Method::Delete.as_str(), "DELETE");
    }
}
