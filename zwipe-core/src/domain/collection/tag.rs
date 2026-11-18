//! The tags a collection row carries, from Archidekt's per-row Tags cell.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Whether `c` is invisible in text yet changes how it reads or matches: a control character,
/// a bidi mark (U+061C, U+200E, U+200F, U+202A to U+202E, U+2066 to U+2069) that can reorder
/// what's around it, or a zero-width space, word joiner or mid-text byte order mark (U+200B,
/// U+2060, U+FEFF) that splits a word without showing. The zero-width joiners U+200C and U+200D
/// stay: some scripts and emoji need them.
#[must_use]
pub fn is_unseen(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{61c}'
                | '\u{200b}'
                | '\u{200e}'
                | '\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'
                | '\u{2066}'..='\u{2069}'
                | '\u{feff}'
        )
}

/// `text` without its [unseen](is_unseen) characters.
#[must_use]
pub fn strip_unseen(text: &str) -> String {
    text.chars().filter(|c| !is_unseen(*c)).collect()
}

/// Why a row's tags were turned down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidCollectionTags {
    /// More than [`CollectionTags::MAX`] distinct tags.
    #[error("more than {max} tags", max = CollectionTags::MAX)]
    TooMany,
    /// A tag longer than [`CollectionTag::MAX_CHARS`] characters.
    #[error("a tag is over {max} characters", max = CollectionTag::MAX_CHARS)]
    TooLong,
    /// A tag with nothing in it once trimmed.
    #[error("an empty tag")]
    Empty,
}

/// One tag: trimmed, [unseen](is_unseen) characters removed, 1 to
/// [`CollectionTag::MAX_CHARS`] characters. Clients show it as text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CollectionTag(String);

impl CollectionTag {
    /// The longest tag, in characters.
    pub const MAX_CHARS: usize = 40;

    /// A tag from `raw`, [unseen](is_unseen) characters removed and trimmed.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCollectionTags::Empty`] when nothing is left, and
    /// [`InvalidCollectionTags::TooLong`] past [`CollectionTag::MAX_CHARS`].
    pub fn new(raw: &str) -> Result<Self, InvalidCollectionTags> {
        let kept = strip_unseen(raw);
        let tag = kept.trim();
        if tag.is_empty() {
            return Err(InvalidCollectionTags::Empty);
        }
        if tag.chars().count() > Self::MAX_CHARS {
            return Err(InvalidCollectionTags::TooLong);
        }
        Ok(Self(tag.to_owned()))
    }

    /// The tag.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for CollectionTag {
    type Error = InvalidCollectionTags;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::new(&raw)
    }
}

impl From<CollectionTag> for String {
    fn from(tag: CollectionTag) -> Self {
        tag.0
    }
}

impl fmt::Display for CollectionTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A row's tags: distinct (ignoring case, the first spelling kept), in the order given, at most
/// [`CollectionTags::MAX`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "Vec<CollectionTag>", into = "Vec<CollectionTag>")]
pub struct CollectionTags(Vec<CollectionTag>);

impl CollectionTags {
    /// The most tags one row holds.
    pub const MAX: usize = 20;

    /// The tags in an export's cell, split on commas. Empty pieces (`a,,b`) are skipped.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCollectionTags::TooLong`] for a tag past
    /// [`CollectionTag::MAX_CHARS`] and [`InvalidCollectionTags::TooMany`] past
    /// [`CollectionTags::MAX`] distinct tags.
    pub fn from_cell(cell: &str) -> Result<Self, InvalidCollectionTags> {
        let mut tags = Self::default();
        for piece in cell.split(',') {
            match CollectionTag::new(piece) {
                Ok(tag) => tags.add(tag)?,
                Err(InvalidCollectionTags::Empty) => {}
                Err(other) => return Err(other),
            }
        }
        Ok(tags)
    }

    /// Adds `tag` unless one like it is already here.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCollectionTags::TooMany`] when a new tag would pass
    /// [`CollectionTags::MAX`].
    pub fn add(&mut self, tag: CollectionTag) -> Result<(), InvalidCollectionTags> {
        let key = tag.0.to_lowercase();
        if self.0.iter().any(|held| held.0.to_lowercase() == key) {
            return Ok(());
        }
        if self.0.len() >= Self::MAX {
            return Err(InvalidCollectionTags::TooMany);
        }
        self.0.push(tag);
        Ok(())
    }

    /// These and `other`'s, merged as [`CollectionTags::add`] does.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCollectionTags::TooMany`] when the union passes [`CollectionTags::MAX`].
    pub fn union(mut self, other: Self) -> Result<Self, InvalidCollectionTags> {
        for tag in other.0 {
            self.add(tag)?;
        }
        Ok(self)
    }

    /// The tags, in order.
    #[must_use]
    pub fn as_slice(&self) -> &[CollectionTag] {
        &self.0
    }

    /// Whether there are none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// How many there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl TryFrom<Vec<CollectionTag>> for CollectionTags {
    type Error = InvalidCollectionTags;

    fn try_from(list: Vec<CollectionTag>) -> Result<Self, Self::Error> {
        let mut tags = Self::default();
        for tag in list {
            tags.add(tag)?;
        }
        Ok(tags)
    }
}

impl From<CollectionTags> for Vec<CollectionTag> {
    fn from(tags: CollectionTags) -> Self {
        tags.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(tags: &CollectionTags) -> Vec<&str> {
        tags.as_slice().iter().map(CollectionTag::as_str).collect()
    }

    #[test]
    fn a_cell_splits_trims_and_dedupes() {
        let tags = CollectionTags::from_cell(" Trade , binder 2,,trade, Ünïcode ").unwrap();
        assert_eq!(strings(&tags), ["Trade", "binder 2", "Ünïcode"]);
        assert!(CollectionTags::from_cell("").unwrap().is_empty());
    }

    #[test]
    fn control_characters_never_reach_a_tag() {
        let tags = CollectionTags::from_cell("bin\u{0}der,\u{1b}[31mred").unwrap();
        assert_eq!(strings(&tags), ["binder", "[31mred"]);
    }

    #[test]
    fn bidi_and_zero_width_marks_never_reach_a_tag() {
        let tags =
            CollectionTags::from_cell("\u{202e}gnirt\u{202c},bin\u{200e}der\u{2066}x\u{2069}")
                .unwrap();
        assert_eq!(strings(&tags), ["gnirt", "binderx"]);
        assert_eq!(strip_unseen("a\u{200f}b\u{202a}c\u{2067}d\u{0}e"), "abcde");
        for mark in ['\u{200b}', '\u{2060}', '\u{feff}', '\u{61c}'] {
            assert_eq!(
                strip_unseen(&format!("Sol{mark}Ring")),
                "SolRing",
                "{:04X}",
                u32::from(mark)
            );
        }
        for joiner in ['\u{200c}', '\u{200d}'] {
            assert!(
                strip_unseen(&format!("a{joiner}b")).contains(joiner),
                "joiners carry meaning and stay"
            );
        }
    }

    #[test]
    fn caps_turn_the_cell_down() {
        let many: Vec<String> = (0..=CollectionTags::MAX).map(|n| format!("t{n}")).collect();
        assert_eq!(
            CollectionTags::from_cell(&many.join(",")),
            Err(InvalidCollectionTags::TooMany)
        );
        let long = "x".repeat(CollectionTag::MAX_CHARS + 1);
        assert_eq!(
            CollectionTags::from_cell(&long),
            Err(InvalidCollectionTags::TooLong)
        );
        assert!(CollectionTag::new(&"é".repeat(CollectionTag::MAX_CHARS)).is_ok());
    }

    #[test]
    fn a_union_keeps_one_of_each() {
        let a = CollectionTags::from_cell("trade,foil").unwrap();
        let b = CollectionTags::from_cell("FOIL,deck").unwrap();
        assert_eq!(strings(&a.union(b).unwrap()), ["trade", "foil", "deck"]);
    }
}
