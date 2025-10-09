use std::str::FromStr;

use chrono::{Duration, NaiveDateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const REFRESH_TOKEN_LIFESPAN: Duration = Duration::days(14);

// =======
//  error
// =======

#[derive(Debug, Error)]
pub enum InvalidRefreshToken {
    #[error("must be 32 characters")]
    Length,
}

// ======
//  main
// ======

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RefreshToken {
    pub value: String,
    pub expires_at: NaiveDateTime,
}

impl RefreshToken {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        let value = hex::encode(bytes);
        let expires_at = Utc::now().naive_utc() + REFRESH_TOKEN_LIFESPAN;
        Self { value, expires_at }
    }
}

pub trait Sha256Hash {
    fn sha256_hash(&self) -> String;
}

impl Sha256Hash for RefreshToken {
    fn sha256_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.value.clone());
        hex::encode(hasher.finalize())
    }
}

impl Sha256Hash for String {
    fn sha256_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.clone());
        hex::encode(hasher.finalize())
    }
}

pub struct UnvalidatedRefreshToken(String);

impl UnvalidatedRefreshToken {
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl FromStr for UnvalidatedRefreshToken {
    type Err = InvalidRefreshToken;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().len() != 32 {
            return Err(InvalidRefreshToken::Length);
        }
        Ok(Self(s.to_string()))
    }
}
