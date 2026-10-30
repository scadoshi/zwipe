//! Validated deck name value object.
//!
//! Ensures deck names meet content and length requirements before storage.

use std::fmt::Display;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::moderation::ContainsBadWord;

/// Errors that occur when constructing an invalid deck name.
#[derive(Debug, Error)]
pub enum InvalidDeckname {
    /// Deck name is empty (minimum 1 character required).
    #[error("deck name minimum length is 1 character")]
    TooShort,
    /// Deck name exceeds 64 characters.
    #[error("deck name maximum length is 64 characters")]
    TooLong,
    /// Deck name contains profanity (content moderation check failed).
    #[error("no naughty bad words please")]
    BadWord,
}

/// Longest name [`DeckName::new`] accepts, counted in characters after trimming.
const MAX_LENGTH: usize = 64;

/// Validated deck name (1-64 characters, no profanity).
///
/// # Validation Rules
///
/// - **Length**: 1-64 characters (inclusive), counted after trimming
/// - **Trimmed**: Leading/trailing whitespace removed
/// - **Content**: No profanity (uses two-tier filtering)
///
/// # Example
///
/// ```rust,ignore
/// let name = DeckName::new("Sultai Control")?;
/// println!("Deck: {}", name.as_str());
/// ```
///
/// # Deserializing
///
/// Deserializing trusts the value, like [`DeckName::from_stored`], because
/// serde only ever sees names the server already accepted: deck profiles in
/// API responses. Request bodies carry a plain `String` that goes through
/// [`DeckName::new`].
#[derive(Debug, Clone, PartialEq)]
pub struct DeckName(String);

impl DeckName {
    /// Creates a new validated deck name.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDeckname`] if:
    /// - Name is empty or only whitespace
    /// - Name exceeds 64 characters
    /// - Name contains profanity
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidDeckname> {
        let name = name.into();
        let name = name.trim();
        if name.is_empty() {
            return Err(InvalidDeckname::TooShort);
        }
        if name.chars().count() > MAX_LENGTH {
            return Err(InvalidDeckname::TooLong);
        }
        if name.contains_bad_word() {
            return Err(InvalidDeckname::BadWord);
        }
        Ok(Self(name.to_string()))
    }

    /// Wraps a deck name that already passed [`DeckName::new`] once, such as
    /// one read from the database or a server response.
    ///
    /// Nothing is checked or trimmed, so the name reads back exactly as
    /// stored. Names saved before the rules tightened (whitespace only, or
    /// with surrounding spaces) must keep loading. New input from a user goes
    /// through [`DeckName::new`].
    #[must_use]
    pub fn from_stored(stored: String) -> Self {
        Self(stored)
    }
}

impl Deref for DeckName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for DeckName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for DeckName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self)
    }
}

impl<'de> Deserialize<'de> for DeckName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(DeckName::from_stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_name_new_accepts_valid_name() {
        assert!(DeckName::new("Got Ya Satya").is_ok());
    }

    #[test]
    fn test_deck_name_new_accepts_single_char() {
        assert!(DeckName::new("A").is_ok());
    }

    #[test]
    fn test_deck_name_new_accepts_exactly_64_chars() {
        assert!(DeckName::new("a".repeat(64)).is_ok());
    }

    #[test]
    fn test_deck_name_new_rejects_empty() {
        assert!(matches!(DeckName::new(""), Err(InvalidDeckname::TooShort)));
    }

    #[test]
    fn test_deck_name_new_rejects_65_chars() {
        assert!(matches!(
            DeckName::new("a".repeat(65)),
            Err(InvalidDeckname::TooLong)
        ));
    }

    #[test]
    fn test_deck_name_new_rejects_whitespace_only() {
        assert!(matches!(
            DeckName::new("   "),
            Err(InvalidDeckname::TooShort)
        ));
    }

    #[test]
    fn test_deck_name_new_stores_trimmed_value() {
        assert_eq!(&*DeckName::new("  My Deck  ").unwrap(), "My Deck");
    }

    #[test]
    fn test_deck_name_new_accepts_max_chars_of_two_byte_chars() {
        assert!(DeckName::new("é".repeat(MAX_LENGTH)).is_ok());
    }

    #[test]
    fn test_deck_name_new_accepts_max_chars_of_four_byte_chars() {
        assert!(DeckName::new("🐉".repeat(MAX_LENGTH)).is_ok());
    }

    #[test]
    fn test_deck_name_new_rejects_one_char_over_max_of_multibyte_chars() {
        assert!(matches!(
            DeckName::new("é".repeat(MAX_LENGTH + 1)),
            Err(InvalidDeckname::TooLong)
        ));
    }

    #[test]
    fn test_deck_name_new_rejects_bad_word() {
        assert!(matches!(
            DeckName::new("fuck"),
            Err(InvalidDeckname::BadWord)
        ));
    }

    #[test]
    fn test_deck_name_new_allows_spaces() {
        assert!(DeckName::new("Got Ya Satya").is_ok());
    }

    #[test]
    fn test_deck_name_from_stored_keeps_value_as_is() {
        assert_eq!(&*DeckName::from_stored("   ".to_string()), "   ");
        assert_eq!(
            &*DeckName::from_stored("  Spaced  ".to_string()),
            "  Spaced  "
        );
    }

    #[test]
    fn test_deck_name_deserialize_keeps_stored_value() {
        let name: DeckName = serde_json::from_str("\"  Spaced  \"").unwrap();
        assert_eq!(&*name, "  Spaced  ");
    }

    #[test]
    fn test_deck_name_deref_returns_inner_value() {
        assert_eq!(&*DeckName::new("Got Ya Satya").unwrap(), "Got Ya Satya");
    }

    #[test]
    fn test_deck_name_display_formats_correctly() {
        assert_eq!(
            DeckName::new("Got Ya Satya").unwrap().to_string(),
            "Got Ya Satya"
        );
    }
}
