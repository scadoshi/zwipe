//! The card lookup's validated inputs: a name to search for and the type words a public caller
//! may send. The lookup is open to signed-out callers, so every value is checked on the way in
//! and bounded well under what the full card search accepts.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The most cards one lookup answers, one per card (oracle id).
pub const LOOKUP_LIMIT: u32 = 30;

/// The most characters a lookup's name keeps; longer than any card name in print.
pub const MAX_LOOKUP_NAME_CHARS: usize = 100;

/// The most values each of a lookup's lists (types, colors, roles) may hold.
pub const MAX_LOOKUP_CHIPS: usize = 5;

/// The most characters a type word holds; longer than any type or subtype in print.
pub const MAX_TYPE_WORD_CHARS: usize = 40;

/// The name a lookup searches for, matched as a case-insensitive substring of a card's name.
/// Trimmed, at most [`MAX_LOOKUP_NAME_CHARS`] characters and free of control characters. It may
/// be empty when the lookup's chips name something to match.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct LookupName(String);

impl LookupName {
    /// The name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether the name is empty, which matches every name.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A lookup name that can't be used: too long, or holding a control character.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidLookupName {
    /// Past [`MAX_LOOKUP_NAME_CHARS`].
    #[error("the name is over {MAX_LOOKUP_NAME_CHARS} characters")]
    TooLong,
    /// A control character, which no card name holds.
    #[error("the name holds a control character")]
    ControlCharacter,
}

impl TryFrom<String> for LookupName {
    type Error = InvalidLookupName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.chars().any(char::is_control) {
            return Err(InvalidLookupName::ControlCharacter);
        }
        if trimmed.chars().count() > MAX_LOOKUP_NAME_CHARS {
            return Err(InvalidLookupName::TooLong);
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl From<LookupName> for String {
    fn from(name: LookupName) -> Self {
        name.0
    }
}

impl fmt::Display for LookupName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One word of a type line a lookup matches, as the filter's type chips send it: `Creature`,
/// `Goblin`, `Urza's`, `Power-Plant`. Trimmed, 1 to [`MAX_TYPE_WORD_CHARS`] characters, of
/// letters, spaces, hyphens and apostrophes only.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CardTypeWord(String);

impl CardTypeWord {
    /// The word as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A type word that can't be used: empty, too long, or holding something besides letters,
/// spaces, hyphens and apostrophes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidCardTypeWord {
    /// Nothing left once trimmed.
    #[error("a type is empty")]
    Empty,
    /// Past [`MAX_TYPE_WORD_CHARS`].
    #[error("a type is over {MAX_TYPE_WORD_CHARS} characters")]
    TooLong,
    /// A character no type word holds.
    #[error("a type holds {0:?}, which no type does")]
    Character(char),
}

impl TryFrom<String> for CardTypeWord {
    type Error = InvalidCardTypeWord;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(InvalidCardTypeWord::Empty);
        }
        if let Some(bad) = trimmed
            .chars()
            .find(|c| !(c.is_alphabetic() || matches!(c, ' ' | '-' | '\'')))
        {
            return Err(InvalidCardTypeWord::Character(bad));
        }
        if trimmed.chars().count() > MAX_TYPE_WORD_CHARS {
            return Err(InvalidCardTypeWord::TooLong);
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl From<CardTypeWord> for String {
    fn from(word: CardTypeWord) -> Self {
        word.0
    }
}

impl fmt::Display for CardTypeWord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(text: &str) -> Result<LookupName, InvalidLookupName> {
        LookupName::try_from(text.to_owned())
    }

    fn word(text: &str) -> Result<CardTypeWord, InvalidCardTypeWord> {
        CardTypeWord::try_from(text.to_owned())
    }

    #[test]
    fn a_name_is_trimmed_and_may_be_empty() {
        assert_eq!(
            name("  Lightning Bolt ").unwrap().as_str(),
            "Lightning Bolt"
        );
        assert!(name("   ").unwrap().is_empty());
        assert_eq!(name("Lim-Dûl's Vault").unwrap().as_str(), "Lim-Dûl's Vault");
    }

    #[test]
    fn a_name_past_the_cap_or_with_a_control_character_is_refused() {
        let at_cap = "a".repeat(MAX_LOOKUP_NAME_CHARS);
        assert!(name(&at_cap).is_ok());
        assert_eq!(name(&format!("{at_cap}a")), Err(InvalidLookupName::TooLong));
        assert_eq!(name("Sol\nRing"), Err(InvalidLookupName::ControlCharacter));
        assert_eq!(
            name("Sol\u{0}Ring"),
            Err(InvalidLookupName::ControlCharacter)
        );
    }

    #[test]
    fn a_type_word_takes_letters_spaces_hyphens_and_apostrophes() {
        for ok in ["Creature", "Urza's", "Power-Plant", "Time Lord", "Æther"] {
            assert_eq!(word(ok).unwrap().as_str(), ok);
        }
        assert_eq!(word(" Goblin ").unwrap().as_str(), "Goblin");
        assert_eq!(word(" "), Err(InvalidCardTypeWord::Empty));
        assert_eq!(word("Goblin%"), Err(InvalidCardTypeWord::Character('%')));
        assert_eq!(word("a_b"), Err(InvalidCardTypeWord::Character('_')));
        let past = "a".repeat(MAX_TYPE_WORD_CHARS + 1);
        assert_eq!(word(&past), Err(InvalidCardTypeWord::TooLong));
    }

    #[test]
    fn both_read_through_serde_with_their_checks() {
        let read: Result<CardTypeWord, _> = serde_json::from_str("\"Goblin;\"");
        assert!(read.is_err());
        let read: LookupName = serde_json::from_str("\" Bolt \"").unwrap();
        assert_eq!(read.as_str(), "Bolt");
    }
}
