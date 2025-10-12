use crate::domain::moderation::ContainsBadWord;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum InvalidUsername {
    #[error("must be at least 3 characters long")]
    TooShort,
    #[error("must not exceed 20 characters")]
    TooLong,
    #[error("cannot contain whitespace")]
    Whitespace,
    #[error("no naughty bad words please")]
    BadWord,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Username(String);

impl Username {
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
