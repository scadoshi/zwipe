use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid color")]
pub struct InvalidColor;

/// stores color information in ScryfallData against various color related fields
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::White => "W".serialize(serializer),
            Self::Blue => "U".serialize(serializer),
            Self::Black => "B".serialize(serializer),
            Self::Red => "R".serialize(serializer),
            Self::Green => "G".serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "w" => Ok(Self::White),
            "u" => Ok(Self::Blue),
            "b" => Ok(Self::Black),
            "r" => Ok(Self::Red),
            "g" => Ok(Self::Green),
            _ => Err(serde::de::Error::custom(format!(
                "{} is an invalid color",
                s
            ))),
        }
    }
}

impl TryFrom<&str> for Color {
    type Error = InvalidColor;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "w" => Ok(Self::White),
            "u" => Ok(Self::Blue),
            "b" => Ok(Self::Black),
            "r" => Ok(Self::Red),
            "g" => Ok(Self::Green),
            _ => Err(InvalidColor),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Colors(Vec<Color>);

impl Serialize for Colors {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Colors {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<Color>::deserialize(deserializer).map(Colors)
    }
}

impl FromIterator<Color> for Colors {
    fn from_iter<T: IntoIterator<Item = Color>>(iter: T) -> Self {
        Colors(iter.into_iter().collect())
    }
}
