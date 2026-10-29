use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error returned when parsing an invalid color string.
#[derive(Debug, Error)]
#[error("invalid color")]
pub struct InvalidColor;

/// Magic: The Gathering's five colors.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// The name Magic players give a color identity: "Colorless", "Mono-blue", a
/// guild ("Azorius"), shard ("Esper") or wedge ("Jeskai"), a four-color name
/// after its missing color ("Glint-Eye" lacks white), or "Five-color".
///
/// Order and repeats in `colors` are ignored: `[Red, White, Blue]` and
/// `[Blue, Red, White, Red]` are both "Jeskai".
#[must_use]
pub fn identity_name(colors: &[Color]) -> &'static str {
    let has = |color: Color| colors.contains(&color);
    match (
        has(Color::White),
        has(Color::Blue),
        has(Color::Black),
        has(Color::Red),
        has(Color::Green),
    ) {
        (false, false, false, false, false) => "Colorless",
        (true, false, false, false, false) => "Mono-white",
        (false, true, false, false, false) => "Mono-blue",
        (false, false, true, false, false) => "Mono-black",
        (false, false, false, true, false) => "Mono-red",
        (false, false, false, false, true) => "Mono-green",
        (true, true, false, false, false) => "Azorius",
        (false, true, true, false, false) => "Dimir",
        (false, false, true, true, false) => "Rakdos",
        (false, false, false, true, true) => "Gruul",
        (true, false, false, false, true) => "Selesnya",
        (true, false, true, false, false) => "Orzhov",
        (false, true, false, true, false) => "Izzet",
        (false, false, true, false, true) => "Golgari",
        (true, false, false, true, false) => "Boros",
        (false, true, false, false, true) => "Simic",
        (true, true, false, false, true) => "Bant",
        (true, true, true, false, false) => "Esper",
        (false, true, true, true, false) => "Grixis",
        (false, false, true, true, true) => "Jund",
        (true, false, false, true, true) => "Naya",
        (true, false, true, false, true) => "Abzan",
        (true, true, false, true, false) => "Jeskai",
        (false, true, true, false, true) => "Sultai",
        (true, false, true, true, false) => "Mardu",
        (false, true, false, true, true) => "Temur",
        (false, true, true, true, true) => "Glint-Eye",
        (true, false, true, true, true) => "Dune-Brood",
        (true, true, false, true, true) => "Ink-Treader",
        (true, true, true, false, true) => "Witch-Maw",
        (true, true, true, true, false) => "Yore-Tiller",
        (true, true, true, true, true) => "Five-color",
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
            _ => Err(serde::de::Error::custom(format!("{s} is an invalid color"))),
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

impl Colors {
    /// Parses a `Vec<String>` of short color codes into `Colors`.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidColor`] for the first name that is not a color code.
    pub fn from_short_names(names: Vec<String>) -> Result<Self, InvalidColor> {
        names
            .into_iter()
            .map(Color::try_from)
            .collect::<Result<Vec<Color>, _>>()
            .map(Colors)
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
    /// Converts all colors to short codes (e.g., `["W", "U", "B"]`).
    pub fn to_short_names(&self) -> Vec<String> {
        self.0.iter().map(Color::to_short_name).collect()
    }

    /// Converts all colors to long names (e.g., `["White", "Blue", "Black"]`).
    pub fn to_long_names(&self) -> Vec<String> {
        self.0.iter().map(Color::to_long_name).collect()
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

#[cfg(test)]
mod tests {
    use Color::{Black, Blue, Green, Red, White};

    use super::*;

    /// Every identity in WUBRG order with its name.
    const NAMED: [(&[Color], &str); 32] = [
        (&[], "Colorless"),
        (&[White], "Mono-white"),
        (&[Blue], "Mono-blue"),
        (&[Black], "Mono-black"),
        (&[Red], "Mono-red"),
        (&[Green], "Mono-green"),
        (&[White, Blue], "Azorius"),
        (&[Blue, Black], "Dimir"),
        (&[Black, Red], "Rakdos"),
        (&[Red, Green], "Gruul"),
        (&[White, Green], "Selesnya"),
        (&[White, Black], "Orzhov"),
        (&[Blue, Red], "Izzet"),
        (&[Black, Green], "Golgari"),
        (&[White, Red], "Boros"),
        (&[Blue, Green], "Simic"),
        (&[White, Blue, Green], "Bant"),
        (&[White, Blue, Black], "Esper"),
        (&[Blue, Black, Red], "Grixis"),
        (&[Black, Red, Green], "Jund"),
        (&[White, Red, Green], "Naya"),
        (&[White, Black, Green], "Abzan"),
        (&[White, Blue, Red], "Jeskai"),
        (&[Blue, Black, Green], "Sultai"),
        (&[White, Black, Red], "Mardu"),
        (&[Blue, Red, Green], "Temur"),
        (&[Blue, Black, Red, Green], "Glint-Eye"),
        (&[White, Black, Red, Green], "Dune-Brood"),
        (&[White, Blue, Red, Green], "Ink-Treader"),
        (&[White, Blue, Black, Green], "Witch-Maw"),
        (&[White, Blue, Black, Red], "Yore-Tiller"),
        (&[White, Blue, Black, Red, Green], "Five-color"),
    ];

    #[test]
    fn identity_name_names_every_identity() {
        for (colors, name) in NAMED {
            assert_eq!(identity_name(colors), name, "{colors:?}");
        }
    }

    #[test]
    fn identity_names_are_distinct() {
        let mut names: Vec<&str> = NAMED.iter().map(|(_, name)| *name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), NAMED.len());
    }

    #[test]
    fn identity_name_ignores_order_and_repeats() {
        for (colors, name) in NAMED {
            let reversed: Vec<Color> = colors.iter().rev().copied().collect();
            assert_eq!(identity_name(&reversed), name, "reversed {colors:?}");
            let doubled: Vec<Color> = reversed.iter().chain(colors).copied().collect();
            assert_eq!(identity_name(&doubled), name, "doubled {colors:?}");
        }
        assert_eq!(identity_name(&[Red, White, Blue]), "Jeskai");
        assert_eq!(identity_name(&[Green, Black, White]), "Abzan");
    }
}
