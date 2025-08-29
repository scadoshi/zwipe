use crate::domain::card::models::scryfall_card::image_uris::ImageUris;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// stores card face data against ScryfallCard
/// against card_faces field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardFace {
    pub artist: Option<String>,
    pub artist_id: Option<Uuid>,
    pub cmc: Option<f32>,
    pub color_indicator: Option<Vec<String>>,
    pub colors: Option<Vec<String>>,
    pub defense: Option<String>,
    pub flavor_text: Option<String>,
    pub illustration_id: Option<Uuid>,
    pub image_uris: Option<ImageUris>,
    pub layout: Option<String>,
    pub loyalty: Option<String>,
    pub mana_cost: String,
    pub name: String,
    pub object: String,
    pub oracle_id: Option<Uuid>,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type_line: Option<String>,
    pub toughness: Option<String>,
    pub type_line: Option<String>,
    pub watermark: Option<String>,
}

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
