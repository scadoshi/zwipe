use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Information about a related card component (other faces, tokens, meld pairs, etc.).
///
/// Used to link cards that work together or are part of the same card object.
///
/// # Component Types
/// - **"combo_piece"**: Meld pair (e.g., Urza + Mishra → Urza and Mishra)
/// - **"token"**: Token created by the card
/// - **"meld_part"**: Part of a meld combination
/// - **"meld_result"**: Result of melding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelatedCard {
    /// Scryfall UUID for the related card.
    pub id: Uuid,
    /// Scryfall object type. Always "related_card".
    pub object: String,
    /// Component relationship type (e.g., "token", "combo_piece", "meld_part").
    pub component: String,
    /// Name of the related card.
    pub name: String,
    /// Type line of the related card (e.g., "Token Creature — Goblin").
    pub type_line: String,
    /// Scryfall API URI for the related card.
    pub uri: String,
}

/// Collection of related cards (wrapper around `Vec<RelatedCard>`).
///
/// Used in [`ScryfallData`](super::ScryfallData) to link multi-part cards,
/// meld pairs, and generated tokens.
///
/// # Examples
/// - **Meld cards**: Bruna + Gisela → Brisela (3 related cards)
/// - **Token creators**: Young Pyromancer → Elemental token (1 related card)
/// - **Split cards**: Fire // Ice (2 related cards for each half)
#[derive(Debug, Clone, PartialEq)]
pub struct AllParts(Vec<RelatedCard>);

impl Serialize for AllParts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AllParts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<RelatedCard>::deserialize(deserializer).map(AllParts)
    }
}
