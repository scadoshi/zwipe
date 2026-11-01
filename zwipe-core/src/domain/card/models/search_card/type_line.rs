//! Reading a card's type line: its front face and the card types on it.

use crate::domain::card::search_card::card_type::CardType;

/// The front face of a type line.
///
/// Scryfall joins a multi-faced card's faces into one `type_line` with
/// `" // "`. Most rules read the front face alone: Westvale Abbey is a Land
/// that transforms into a legendary creature, and it is not a creature in a
/// deck. A single-faced type line has no separator and comes back whole.
#[must_use]
pub fn front_face(type_line: &str) -> &str {
    type_line
        .split_once(" // ")
        .map_or(type_line, |(front, _)| front)
}

/// Whether the front face of `type_line` has `card_type` as one of its
/// words, so "Artifact Creature" has both Artifact and Creature. A whole
/// word must match: "Snow Artifacts" is not an Artifact. `None` has no type.
#[must_use]
pub fn has_type(type_line: Option<&str>, card_type: CardType) -> bool {
    let word = card_type.to_string();
    type_line.is_some_and(|line| front_face(line).split_whitespace().any(|w| w == word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn front_face_is_the_part_before_the_separator() {
        assert_eq!(front_face("Land // Legendary Creature"), "Land");
        assert_eq!(
            front_face("Legendary Creature — Elf"),
            "Legendary Creature — Elf"
        );
        assert_eq!(front_face(""), "");
    }

    #[test]
    fn has_type_reads_every_word_of_the_front_face() {
        let line = Some("Legendary Artifact Creature — Golem");
        assert!(has_type(line, CardType::Artifact));
        assert!(has_type(line, CardType::Creature));
        assert!(!has_type(line, CardType::Enchantment));
        let modal = Some("Sorcery // Land");
        assert!(has_type(modal, CardType::Sorcery));
        assert!(
            !has_type(modal, CardType::Land),
            "the back face is not read"
        );
        assert!(!has_type(None, CardType::Creature));
        assert!(
            !has_type(Some("Snow Artifacts"), CardType::Artifact),
            "a whole word, not a prefix"
        );
    }
}
