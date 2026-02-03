//! Username value object and validation.
//!
//! This module provides the [`Username`] type, which ensures all usernames
//! meet platform requirements for length, content, and appropriateness.

use crate::domain::moderation::ContainsBadWord;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

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
    #[error("must be at least 3 characters long")]
    TooShort,

    /// Username exceeds 20 characters.
    ///
    /// Maximum length ensures usernames fit in UI components and
    /// database indexes.
    #[error("must not exceed 20 characters")]
    TooLong,

    /// Username contains whitespace (spaces, tabs, newlines).
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
/// This value object wraps a string and guarantees that any instance contains
/// a valid username. The string is trimmed and validated on construction.
///
/// # Validation Rules
///
/// - **Length**: 3-20 characters
/// - **No Whitespace**: No spaces, tabs, or newlines
/// - **No Profanity**: Checked against bad word dictionary
/// - **Trimmed**: Leading/trailing whitespace removed
///
/// # Immutability
///
/// Once created, a `Username` cannot be modified (no `pub` fields, no setters).
/// This ensures validation rules cannot be bypassed.
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::user::models::username::Username;
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
    /// * `raw` - The raw username string to validate
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
    pub fn new(raw: &str) -> Result<Self, InvalidUsername> {
        let trimmed = raw.trim();

        if trimmed.contains_bad_word() {
            return Err(InvalidUsername::BadWord);
        }

        if trimmed.chars().any(|c| c.is_whitespace()) {
            return Err(InvalidUsername::Whitespace);
        }

        if trimmed.len() < 3 {
            return Err(InvalidUsername::TooShort);
        }

        if trimmed.len() > 20 {
            return Err(InvalidUsername::TooLong);
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Returns the username as a string slice.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let username = Username::new("alice")?;
    /// assert_eq!(username.as_str(), "alice");
    /// ```
    pub fn as_str(&self) -> &str {
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
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Username::new(raw.as_str()).map_err(serde::de::Error::custom)
    }
}
