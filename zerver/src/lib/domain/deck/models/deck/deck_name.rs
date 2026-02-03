//! Validated deck name value object.
//!
//! Ensures deck names meet content and length requirements before storage.

use crate::domain::moderation::ContainsBadWord;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
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
    pub fn new(name: &str) -> Result<Self, InvalidDeckname> {
        if name.is_empty() {
            return Err(InvalidDeckname::TooShort);
        }

        if name.len() > 64 {
            return Err(InvalidDeckname::TooLong);
        }

        if name.contains_bad_word() {
            return Err(InvalidDeckname::BadWord);
        }

        Ok(Self(name.to_string()))
    }

    /// Returns the deck name as a string slice.
    pub fn as_str(&self) -> &str {
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
        serializer.serialize_str(self.as_str())
    }
}
