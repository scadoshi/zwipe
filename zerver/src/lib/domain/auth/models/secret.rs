//! Plaintext secrets that survive a `Debug` print.

use std::fmt::{Debug, Display, Formatter};

/// Plaintext the user typed, held only long enough to verify it against a
/// stored hash.
///
/// Unvalidated on purpose. A password set under an older policy must still
/// authenticate, so verification paths take this and
/// [`Password`](super::password::Password) is reserved for values that are new
/// and must meet current policy.
///
/// # Example
///
/// ```rust,ignore
/// let secret = Secret::new(&body.password);
/// assert_eq!(format!("{secret:?}"), "Secret(REDACTED)");
/// stored_hash.verify(secret.read())?;
/// ```
#[derive(Clone)]
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

    /// Anything authenticates: the stored hash is the only judge.
    #[test]
    fn secrets_are_not_policy_checked() {
        assert_eq!(Secret::new("x").read(), "x");
        assert_eq!(Secret::new("").read(), "");
    }
}
