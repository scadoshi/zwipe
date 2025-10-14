use crate::domain::moderation::ContainsBadWord;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidDeckname {
    #[error("deck name minimum length is 1 character")]
    TooShort,
    #[error("deck name maximum length is 64 characters")]
    TooLong,
    #[error("no naughty bad words please")]
    BadWord,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DeckName(String);

impl DeckName {
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
