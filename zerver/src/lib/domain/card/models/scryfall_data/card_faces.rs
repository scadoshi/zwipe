use crate::domain::card::models::scryfall_data::image_uris::ImageUris;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Data for a single face of a multi-faced card.
///
/// Used for double-faced cards (DFCs), transform cards, modal DFCs, split cards, etc.
/// Each face has its own properties like name, mana cost, oracle text, and images.
///
/// # Examples of Multi-Faced Cards
/// - **Transform DFCs**: Delver of Secrets // Insectile Aberration
/// - **Modal DFCs**: Valki, God of Lies // Tibalt, Cosmic Impostor
/// - **Split Cards**: Fire // Ice
/// - **Adventure Cards**: Bonecrusher Giant (creature) // Stomp (instant)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardFace {
    /// Artist name for this face. `None` if not attributed.
    pub artist: Option<String>,
    /// Scryfall UUID for the artist. `None` if not in database.
    pub artist_id: Option<Uuid>,
    /// Converted mana cost for this face. `None` if no mana cost.
    pub cmc: Option<f32>,
    /// Color indicator (colored dot) for this face. `None` if no indicator.
    pub color_indicator: Option<Vec<String>>,
    /// Colors based on mana cost/indicator. `None` if colorless.
    pub colors: Option<Vec<String>>,
    /// Defense value for battle cards. `None` for non-battles.
    pub defense: Option<String>,
    /// Flavor text for this face. `None` if no flavor text.
    pub flavor_text: Option<String>,
    /// Scryfall UUID for illustration. `None` if not cataloged.
    pub illustration_id: Option<Uuid>,
    /// Image URIs at various resolutions for this face.
    pub image_uris: Option<ImageUris>,
    /// Layout type for this face (usually matches parent card).
    pub layout: Option<String>,
    /// Loyalty value for planeswalker faces. `None` for non-planeswalkers.
    pub loyalty: Option<String>,
    /// Mana cost in Scryfall notation (e.g., "{2}{R}{R}").
    pub mana_cost: String,
    /// Name of this card face.
    pub name: String,
    /// Scryfall object type. Always "card_face".
    pub object: String,
    /// Oracle ID shared across all printings. `None` for tokens.
    pub oracle_id: Option<Uuid>,
    /// Oracle rules text for this face. `None` if no text.
    pub oracle_text: Option<String>,
    /// Power value for creature faces. `None` for non-creatures.
    pub power: Option<String>,
    /// Printed name in card's language (non-English). `None` if English.
    pub printed_name: Option<String>,
    /// Printed text in card's language. `None` if English.
    pub printed_text: Option<String>,
    /// Printed type line in card's language. `None` if English.
    pub printed_type_line: Option<String>,
    /// Toughness value for creature faces. `None` for non-creatures.
    pub toughness: Option<String>,
    /// Type line (e.g., "Creature — Human Wizard"). `None` is rare.
    pub type_line: Option<String>,
    /// Watermark name. `None` if no watermark.
    pub watermark: Option<String>,
}

/// Collection of card faces (wrapper around `Vec<CardFace>`).
///
/// Used in [`ScryfallData`](super::ScryfallData) for multi-faced cards.
#[derive(Debug, Clone, PartialEq)]
pub struct CardFaces(Vec<CardFace>);

impl Serialize for CardFaces {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CardFaces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Vec::<CardFace>::deserialize(deserializer).map(CardFaces)
    }
}
