//! Deck validation warning value object.

use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// A deck-building warning message (informational, not blocking).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeckWarning(String);

impl DeckWarning {
    /// Creates a new deck warning.
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Deref for DeckWarning {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for DeckWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
