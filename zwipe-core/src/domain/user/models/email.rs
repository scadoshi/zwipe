//! Email value object and validation.
//!
//! This module provides the [`Email`] type, a validated email address that satisfies
//! a deliberately conservative subset of RFC 5321 chosen to maximize interoperability
//! with downstream consumers (DNS-resolvable domains only, no exotic syntax).
//!
//! Use this type everywhere — never construct an [`email_address::EmailAddress`]
//! directly. The newtype is what makes the strict ruleset enforceable; a free
//! parser function could always be bypassed by calling the underlying crate's
//! constructors. Outbound adapters that need a `&str` for transport call
//! [`Email::as_ref`].

use email_address::{EmailAddress, Options};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref, str::FromStr};

/// Validation error when constructing an [`Email`].
///
/// Wraps the underlying parser error from the `email_address` crate so callers
/// don't need to depend on that crate directly.
pub type InvalidEmail = email_address::Error;

/// A validated email address whose textual form is the canonical
/// `local-part@sub.domain.tld` shape — nothing more, nothing less.
///
/// # Validation rules
///
/// Stricter than the default `email_address` parse:
/// - Must have a TLD (`user@example.com` ok, `user@apple` rejected).
/// - No domain literals (`user@[1.2.3.4]` rejected — we require a resolvable
///   hostname).
/// - No display-text wrappers (`Name <user@example.com>` rejected — display
///   names are an envelope concern, not part of the address itself).
///
/// These rules exist so that every [`Email`] is unambiguously routable by any
/// reasonable downstream consumer (DNS, SMTP, third-party delivery APIs)
/// without re-validation or transformation.
///
/// # Immutability
///
/// `Email` has no public fields and no setters. Once constructed, it is
/// guaranteed valid. To change it, parse a new one.
///
/// # Why a newtype, not a helper function
///
/// A `fn parse_email(...) -> EmailAddress` would not stop a future caller from
/// reaching for `EmailAddress::from_str` and skipping the strict options. By
/// using a separate type and never exposing the inner [`EmailAddress`], the
/// type system enforces the rule.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Email(EmailAddress);

impl Email {
    /// Creates a new validated email.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidEmail`] if the input fails any of the strict rules
    /// described in the type-level docs.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, InvalidEmail> {
        let parsed = EmailAddress::parse_with_options(
            raw.as_ref(),
            Options::default()
                .with_required_tld()
                .without_domain_literal()
                .without_display_text(),
        )?;
        Ok(Self(parsed))
    }
}

impl FromStr for Email {
    type Err = InvalidEmail;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Deref for Email {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Email> for String {
    fn from(email: Email) -> Self {
        email.0.into()
    }
}

impl Serialize for Email {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Email::new(raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_standard_address() {
        assert!(Email::new("user@example.com").is_ok());
    }

    #[test]
    fn accepts_plus_addressing() {
        assert!(Email::new("user+tag@example.com").is_ok());
    }

    #[test]
    fn rejects_missing_tld() {
        assert!(Email::new("user@apple").is_err());
    }

    #[test]
    fn rejects_domain_literal() {
        assert!(Email::new("user@[1.2.3.4]").is_err());
    }

    #[test]
    fn rejects_display_text_wrapper() {
        assert!(Email::new("Name <user@example.com>").is_err());
    }

    #[test]
    fn rejects_missing_at() {
        assert!(Email::new("not-an-email").is_err());
    }

    #[test]
    fn display_returns_address_only() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(email.to_string(), "user@example.com");
    }

    #[test]
    fn serializes_as_plain_string() {
        let email = Email::new("user@example.com").unwrap();
        let json = serde_json::to_string(&email).unwrap();
        assert_eq!(json, "\"user@example.com\"");
    }

    #[test]
    fn round_trips_through_serde() {
        let email = Email::new("user@example.com").unwrap();
        let json = serde_json::to_string(&email).unwrap();
        let decoded: Email = serde_json::from_str(&json).unwrap();
        assert_eq!(email, decoded);
    }

    #[test]
    fn deserialize_rejects_invalid_address() {
        let result: Result<Email, _> = serde_json::from_str("\"user@apple\"");
        assert!(result.is_err());
    }
}
