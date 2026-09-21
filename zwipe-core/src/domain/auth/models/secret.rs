//! Plaintext secrets that survive a `Debug` print.

use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

/// Plaintext the user typed, held only long enough to send or verify it.
///
/// Serializes and deserializes as a bare string, so the value crosses the wire
/// exactly as a `String` would. `Debug` and `Display` redact it instead, which
/// keeps it out of logs, panic messages, and crash reports on both the client
/// and the server.
///
/// Unvalidated on purpose. A password set under an older policy must still
/// authenticate, so verification paths take this and
/// [`validate`](super::password::validate) gates values that are new and must
/// meet current policy.
///
/// # Example
///
/// ```
/// # use zwipe_core::domain::auth::models::secret::Secret;
/// let secret = Secret::new("hunter2");
/// assert_eq!(format!("{secret:?}"), "Secret(REDACTED)");
/// assert_eq!(serde_json::to_string(&secret).unwrap(), "\"hunter2\"");
/// assert_eq!(secret.read(), "hunter2");
/// ```
#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    /// Wraps a plaintext secret. No validation, by design (see the type docs).
    pub fn new(raw: impl AsRef<str>) -> Self {
        Self(raw.as_ref().to_string())
    }

    /// Returns the plaintext.
    ///
    /// # Security Warning
    ///
    /// The only way past the redacted formatters. Use it for verification and
    /// hashing, never for logging or an API response.
    pub fn read(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Secret {
    fn from(raw: &str) -> Self {
        Self::new(raw)
    }
}

/// Never derive this: the derive prints the plaintext, and every struct
/// holding a `Secret` inherits that through its own `Debug`.
/// [`read`](Secret::read) is the only way to the secret.
impl Debug for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(REDACTED)")
    }
}

/// Redacted, as with `Debug` above.
impl Display for Secret {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("REDACTED")
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// Catches a re-derived `Debug` or `Display`, either of which prints the
    /// plaintext.
    #[test]
    fn formatting_a_secret_never_reveals_it() {
        let plaintext = "OldWeakPassword";
        let secret = Secret::new(plaintext);

        let debug = format!("{secret:?}");
        let display = format!("{secret}");

        assert!(!debug.contains(plaintext), "Debug leaked: {debug}");
        assert!(!display.contains(plaintext), "Display leaked: {display}");
        assert_eq!(secret.read(), plaintext, "read() is still the way in");
    }

    /// The redaction must never reach the wire: a `Secret` field has to look
    /// exactly like the `String` field it replaced, in both directions, or
    /// shipped clients break.
    #[test]
    fn a_secret_is_wire_identical_to_a_string() {
        #[derive(Serialize)]
        struct Old {
            password: String,
        }
        #[derive(Serialize)]
        struct New {
            password: Secret,
        }

        let old = serde_json::to_string(&Old {
            password: "hunter2".to_string(),
        })
        .unwrap();
        let new = serde_json::to_string(&New {
            password: Secret::new("hunter2"),
        })
        .unwrap();
        assert_eq!(old, new);

        #[derive(Deserialize)]
        struct Incoming {
            password: Secret,
        }
        let from_shipped_client = r#"{"password":"hunter2"}"#;
        let parsed: Incoming = serde_json::from_str(from_shipped_client).unwrap();
        assert_eq!(parsed.password.read(), "hunter2");
    }

    /// Anything authenticates: the stored hash is the only judge.
    #[test]
    fn secrets_are_not_policy_checked() {
        assert_eq!(Secret::new("x").read(), "x");
        assert_eq!(Secret::new("").read(), "");
    }
}
