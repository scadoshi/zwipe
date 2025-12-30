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

impl Color {
    pub fn to_long_name(&self) -> String {
        let s = match self {
            Self::White => "White",
            Self::Blue => "Blue",
            Self::Black => "Black",
            Self::Red => "Red",
            Self::Green => "Green",
        };
        s.to_string()
    }

    pub fn to_short_name(&self) -> String {
        let s = match self {
            Self::White => "W",
            Self::Blue => "U",
            Self::Black => "B",
            Self::Red => "R",
            Self::Green => "G",
        };
        s.to_string()
    }

    pub fn all() -> [Self; 5] {
        [Self::White, Self::Blue, Self::Black, Self::Red, Self::Green]
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_long_name())
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_short_name().serialize(serializer)
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

impl TryFrom<String> for Color {
    type Error = InvalidColor;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Colors(Vec<Color>);

impl Colors {
    pub fn to_short_names(&self) -> Vec<String> {
        self.0.iter().map(|c| c.to_short_name()).collect()
    }

    pub fn to_long_names(&self) -> Vec<String> {
        self.0.iter().map(|c| c.to_long_name()).collect()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, color: &Color) -> bool {
        self.0.contains(color)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Color> {
        self.0.iter()
    }

    pub fn as_slice(&self) -> &[Color] {
        self.0.as_slice()
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

impl<I> From<I> for Colors
where
    I: IntoIterator<Item = Color>,
{
    fn from(value: I) -> Self {
        value.into_iter().collect()
    }
}

impl Serialize for Colors {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl FromIterator<Color> for Colors {
    fn from_iter<T: IntoIterator<Item = Color>>(iter: T) -> Self {
        Colors(iter.into_iter().collect())
    }
}
