/// An object containing daily price information for this card,
/// including usd, usd_foil, usd_etched, eur, eur_foil,
/// eur_etched, and tix prices, as strings.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prices {
    pub usd: Option<String>,
    pub usd_foil: Option<String>,
    pub usd_etched: Option<String>,
    pub eur: Option<String>,
    pub eur_foil: Option<String>,
    pub eur_etched: Option<String>,
    pub tix: Option<String>,
}

/// An object describing the legality of this 
/// card across play formats.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Legalities {
    pub standard: Option<LegalityKind>,
    pub future: Option<LegalityKind>,
    pub historic: Option<LegalityKind>,
    pub timeless: Option<LegalityKind>,
    pub gladiator: Option<LegalityKind>,
    pub pioneer: Option<LegalityKind>,
    pub modern: Option<LegalityKind>,
    pub legacy: Option<LegalityKind>,
    pub pauper: Option<LegalityKind>,
    pub vintage: Option<LegalityKind>,
    pub penny: Option<LegalityKind>,
    pub commander: Option<LegalityKind>,
    pub oathbreaker: Option<LegalityKind>,
    pub standardbrawl: Option<LegalityKind>,
    pub brawl: Option<LegalityKind>,
    pub alchemy: Option<LegalityKind>,
    pub paupercommander: Option<LegalityKind>,
    pub duel: Option<LegalityKind>,
    pub oldschool: Option<LegalityKind>,
    pub premodern: Option<LegalityKind>,
    pub predh: Option<LegalityKind>,
    pub explorer: Option<LegalityKind>,
    pub historicbrawl: Option<LegalityKind>,
}

/// Possible legality states for a format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LegalityKind {
    #[default]
    Legal,
    NotLegal,
    Restricted,
    Banned,
}

/// To be stored against various card objects
/// against the "image_uris" field usually
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUris {
    pub small: Option<String>,
    pub normal: Option<String>,
    pub large: Option<String>,
    pub png: Option<String>,
    pub border_crop: Option<String>,
    pub art_crop: Option<String>,
}

/// To be stored against card
/// against the "card_faces" field
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// To be stored against card
/// against the "all_parts" field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedCard {
    pub id: Uuid,
    pub object: String,
    pub component: String,
    pub name: String,
    pub type_line: String,
    pub uri: String,
}
