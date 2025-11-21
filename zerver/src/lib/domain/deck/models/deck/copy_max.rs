use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("must be standard (4) or singleton (1)")]
pub struct InvalidCopyMax;

#[derive(Debug, Clone, PartialEq)]
pub struct CopyMax(i32);

impl CopyMax {
    pub fn new(max: i32) -> Result<Self, InvalidCopyMax> {
        if ![1, 4].contains(&max) {
            return Err(InvalidCopyMax);
        }

        Ok(Self(max))
    }

    pub fn max(&self) -> i32 {
        self.0
    }

    pub fn singleton() -> Self {
        Self(1)
    }

    pub fn standard() -> Self {
        Self(4)
    }
}

impl Serialize for CopyMax {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.max().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CopyMax {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let max: i32 = serde::Deserialize::deserialize(deserializer)?;
        let card_copy_max = CopyMax::new(max).map_err(serde::de::Error::custom)?;
        Ok(card_copy_max)
    }
}
