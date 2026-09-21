//! One description per API call: method, path, auth, body and response type.
//!
//! An endpoint is a value, so paths that interpolate an id carry it
//! (`GetDeck(deck_id)`). Clients drive every call from this, which is what
//! keeps method and path from drifting apart: `/api/deck` is a POST to create
//! and a GET to list, and only this trait can say which.
//!
//! Transport lives in the client, not here; core stays free of reqwest.

use serde::de::DeserializeOwned;
use serde_json::Value;

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
/// `Response` is `()` for endpoints that answer 204: an empty body decodes as
/// `null`, which is exactly what `()` deserializes from.
pub trait Endpoint {
    /// HTTP method.
    const METHOD: Method;
    /// Whether the call carries a bearer token.
    const AUTH: bool = true;
    /// Decoded success body.
    type Response: DeserializeOwned;

    /// Absolute path, ids interpolated.
    fn path(&self) -> String;

    /// JSON request body, or `None` to send none.
    ///
    /// Bodyless calls must return `None`: sending `null` on a GET would put a
    /// body on the wire where shipped clients send none.
    fn body(&self) -> Option<Value> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 204 contract: an empty body is read as `null`, and `()`
    /// deserializes from `null`. Endpoints returning no content depend on it.
    #[test]
    fn unit_decodes_from_null() {
        let decoded: () = serde_json::from_str("null").unwrap();
        assert_eq!(decoded, ());
    }

    #[test]
    fn methods_spell_themselves() {
        assert_eq!(Method::Get.as_str(), "GET");
        assert_eq!(Method::Delete.as_str(), "DELETE");
    }
}
