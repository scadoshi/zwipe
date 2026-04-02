//! Validated deck name value object.
//!
//! Ensures deck names meet content and length requirements before storage.

use crate::domain::moderation::ContainsBadWord;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref};
use thiserror::Error;

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

/// Validated deck name (1-64 characters, no profanity).
///
/// # Validation Rules
///
/// - **Length**: 1-64 characters (inclusive)
/// - **Content**: No profanity (uses two-tier filtering)
///
/// # Example
///
/// ```rust,ignore
/// let name = DeckName::new("Sultai Control")?;
/// println!("Deck: {}", name.as_str());
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeckName(String);

impl DeckName {
    /// Creates a new validated deck name.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDeckname`] if:
    /// - Name is empty
    /// - Name exceeds 64 characters
    /// - Name contains profanity
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidDeckname> {
        let name = name.into();
        if name.is_empty() {
            return Err(InvalidDeckname::TooShort);
        }
        if name.len() > 64 {
            return Err(InvalidDeckname::TooLong);
        }
        if name.contains_bad_word() {
            return Err(InvalidDeckname::BadWord);
        }
        Ok(Self(name))
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
