use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
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
            "w" | "white" => Ok(Self::White),
            "u" | "blue" => Ok(Self::Blue),
            "b" | "black" => Ok(Self::Black),
            "r" | "red" => Ok(Self::Red),
            "g" | "green" => Ok(Self::Green),
            _ => Err(serde::de::Error::custom(format!("invalid color: {}", s))),
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
