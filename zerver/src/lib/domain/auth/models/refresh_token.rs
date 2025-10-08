use std::str::FromStr;

use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

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
pub struct RefreshToken([u8; 32]);

impl RefreshToken {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.0);
        hex::encode(hasher.finalize())
    }
}

impl FromStr for RefreshToken {
    type Err = InvalidRefreshToken;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let validated: [u8; 32] = s
            .as_bytes()
            .try_into()
            .map_err(|_| InvalidRefreshToken::Length)?;
        Ok(Self(validated))
    }
}
