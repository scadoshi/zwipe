//! Username value object and validation.
//!
//! This module provides the [`Username`] type, which ensures all usernames
//! meet platform requirements for length, content, and appropriateness.

use std::fmt::Display;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::moderation::ContainsBadWord;

/// Fewest characters a trimmed username may have.
const MIN_LENGTH: usize = 3;

/// Most characters a trimmed username may have.
const MAX_LENGTH: usize = 20;

/// Zero-width characters that `char::is_whitespace` misses but that render as
/// nothing, so a username containing one would look like another user's.
const ZERO_WIDTH: [char; 5] = ['\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}'];

/// Validation errors when creating a username.
///
/// Usernames must meet multiple requirements to ensure they are:
/// - Identifiable (not too short)
/// - Displayable (not too long for UI)
/// - Appropriate (no profanity)
/// - Valid (no whitespace that causes parsing issues)
#[derive(Debug, Error, Clone)]
pub enum InvalidUsername {
    /// Username is shorter than 3 characters.
    ///
    /// Minimum length ensures usernames are meaningful and reduces
    /// namespace collisions.
    #[error("must be at least {MIN_LENGTH} characters long")]
    TooShort,

    /// Username exceeds 20 characters.
    ///
    /// Maximum length ensures usernames fit in UI components and
    /// database indexes.
    #[error("must not exceed {MAX_LENGTH} characters")]
    TooLong,

    /// Username contains whitespace (spaces, tabs, newlines) or a zero-width
    /// character.
    ///
    /// Whitespace is disallowed to prevent:
    /// - Confusion with display formatting
    /// - URL encoding issues
    /// - Leading/trailing space tricks
    #[error("cannot contain whitespace")]
    Whitespace,

    /// Username contains profanity or inappropriate content.
    ///
    /// Checked against a dictionary of banned words and slurs to
    /// maintain a respectful platform.
    #[error("no naughty bad words please")]
    BadWord,
}

/// A validated username that meets all platform requirements.
///
/// [`Username::new`] trims and validates new input. Stored usernames come back
/// through [`Username::from_stored`] or serde without a second check, so a
/// name accepted under older rules still loads.
///
/// # Validation Rules
///
/// - **Length**: 3-20 characters
/// - **No Whitespace**: No spaces, tabs, newlines, or zero-width characters
/// - **No Profanity**: Checked against bad word dictionary
/// - **Trimmed**: Leading/trailing whitespace removed
///
/// # Immutability
///
/// Once created, a `Username` cannot be modified (no `pub` fields, no setters).
///
/// # Deserializing
///
/// Deserializing trusts the value, like [`Username::from_stored`], because
/// serde only ever sees usernames the server already accepted: JWT claims, API
/// responses and the app's saved session. Request bodies carry a plain `String` that
/// goes through [`Username::new`].
///
/// # Example
///
/// ```rust,ignore
/// use zwipe_core::domain::user::username::Username;
///
/// // Valid usernames
/// let username = Username::new("alice")?;
/// let username = Username::new("Bob123")?;
/// let username = Username::new("user_name")?;
///
/// // Invalid usernames
/// assert!(matches!(
///     Username::new("ab"),
///     Err(InvalidUsername::TooShort)
/// ));
/// assert!(matches!(
///     Username::new("user name"),
///     Err(InvalidUsername::Whitespace)
/// ));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Username(String);

impl Username {
    /// Creates a new validated username.
    ///
    /// The input is trimmed and validated against all requirements. If any
    /// validation fails, an error is returned indicating which rule failed.
    ///
    /// # Arguments
    ///
    /// * `raw`: the raw username string to validate
    ///
    /// # Errors
    ///
    /// Returns [`InvalidUsername`] if the username:
    /// - Is shorter than 3 characters
    /// - Is longer than 20 characters
    /// - Contains whitespace
    /// - Contains profanity or inappropriate content
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let username = Username::new("alice")?;
    /// println!("Username: {}", username);
    /// ```
    pub fn new(raw: impl AsRef<str>) -> Result<Self, InvalidUsername> {
        let trimmed = raw.as_ref().trim();

        if trimmed.contains_bad_word() {
            return Err(InvalidUsername::BadWord);
        }

        if trimmed
            .chars()
            .any(|c| c.is_whitespace() || ZERO_WIDTH.contains(&c))
        {
            return Err(InvalidUsername::Whitespace);
        }

        let length = trimmed.chars().count();

        if length < MIN_LENGTH {
            return Err(InvalidUsername::TooShort);
        }

        if length > MAX_LENGTH {
            return Err(InvalidUsername::TooLong);
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Wraps a username that already passed [`Username::new`] once, such as
    /// one read from the database, a JWT or a server response.
    ///
    /// Nothing is checked or trimmed. The rules can tighten after a name was
    /// accepted (a word joins the ban list, say), and an existing account must
    /// keep loading when they do. New input from a user goes through
    /// [`Username::new`].
    #[must_use]
    pub fn from_stored(stored: String) -> Self {
        Self(stored)
    }
}

impl Deref for Username {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Username {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self)
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Username::from_stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_new_rejects_too_short() {
        assert!(matches!(
            Username::new("ab"),
            Err(InvalidUsername::TooShort)
        ));
    }

    #[test]
    fn test_username_new_accepts_minimum_length() {
        assert!(matches!(Username::new("abc"), Ok(Username(_))));
    }

    #[test]
    fn test_username_new_rejects_too_long() {
        assert!(matches!(
            Username::new("abcdefghijklmnopqrstu"),
            Err(InvalidUsername::TooLong)
        ));
    }

    #[test]
    fn test_username_new_accepts_maximum_length() {
        assert!(matches!(
            Username::new("abcdefghijklmnopqrst"),
            Ok(Username(_))
        ));
    }

    #[test]
    fn test_username_new_rejects_two_multibyte_chars_as_too_short() {
        assert!(matches!(
            Username::new("éé"),
            Err(InvalidUsername::TooShort)
        ));
    }

    #[test]
    fn test_username_new_accepts_max_chars_of_two_byte_chars() {
        assert!(Username::new("é".repeat(MAX_LENGTH)).is_ok());
    }

    #[test]
    fn test_username_new_rejects_one_char_over_max_of_multibyte_chars() {
        assert!(matches!(
            Username::new("é".repeat(MAX_LENGTH + 1)),
            Err(InvalidUsername::TooLong)
        ));
    }

    #[test]
    fn test_username_new_rejects_zero_width_chars() {
        for zw in ['\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}'] {
            assert!(
                matches!(
                    Username::new(format!("scadoshi{zw}")),
                    Err(InvalidUsername::Whitespace)
                ),
                "{zw:?} should be rejected"
            );
        }
    }

    #[test]
    fn test_username_new_rejects_internal_whitespace() {
        assert!(matches!(
            Username::new("abc 123"),
            Err(InvalidUsername::Whitespace)
        ));
    }

    #[test]
    fn test_username_new_rejects_bad_word() {
        assert!(matches!(
            Username::new("fuck"),
            Err(InvalidUsername::BadWord)
        ));
    }

    #[test]
    fn test_username_new_trims_surrounding_whitespace() {
        assert!(matches!(Username::new("   abc    "), Ok(Username(_))));
    }

    #[test]
    fn test_username_as_str_returns_trimmed_value() {
        let username = Username::new("  abc  ").unwrap();
        assert_eq!(&*username, "abc");
    }

    #[test]
    fn test_username_display_formats_correctly() {
        let username = Username::new("  abc  ").unwrap();
        assert_eq!(username.to_string(), "abc");
    }

    #[test]
    fn test_username_deserialize_keeps_banned_word() {
        let username: Username = serde_json::from_str("\"fuck\"").unwrap();
        assert_eq!(&*username, "fuck");
    }

    #[test]
    fn test_username_from_stored_keeps_value_as_is() {
        assert_eq!(&*Username::from_stored(" fuck ".to_string()), " fuck ");
    }

    #[test]
    fn test_username_serialization_round_trip() {
        let username = Username::new("abc").unwrap();
        let json = serde_json::to_string(&username).unwrap();
        let deserialized: Username = serde_json::from_str(&json).unwrap();
        assert_eq!(username, deserialized);
    }
}
