use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error returned when parsing an invalid color string.
#[derive(Debug, Error)]
#[error("invalid color")]
pub struct InvalidColor;

/// Magic: The Gathering's five colors.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

impl Color {
    /// Returns the full color name (e.g., "White", "Blue").
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

    /// Returns the single-letter color code (e.g., "W", "U", "B", "R", "G").
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

    /// Returns all five colors in WUBRG order.
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

/// Collection of card colors.
///
/// Empty collection means colorless.
#[derive(Debug, Clone, PartialEq)]
pub struct Colors(Vec<Color>);

impl std::ops::Deref for Colors {
    type Target = [Color];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Colors {
    /// Converts all colors to short codes (e.g., ["W", "U", "B"]).
    pub fn to_short_names(&self) -> Vec<String> {
        self.0.iter().map(|c| c.to_short_name()).collect()
    }

    /// Converts all colors to long names (e.g., ["White", "Blue", "Black"]).
    pub fn to_long_names(&self) -> Vec<String> {
        self.0.iter().map(|c| c.to_long_name()).collect()
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
