/// Multi-part card relationships (split cards, double-faced cards, etc.).
pub mod all_parts;
/// Card face data for double-faced/transform/modal cards.
pub mod card_faces;
/// Color and color identity types and conversions.
pub mod colors;
/// Get Scryfall data operations.
pub mod get_scryfall_data;
/// Card image URIs at various resolutions.
pub mod image_uris;
/// Format legality status (Standard, Modern, Commander, etc.).
pub mod legalities;
/// Card pricing data from various sources.
pub mod prices;
/// Rarity types (Common, Uncommon, Rare, Mythic).
pub mod rarity;

use all_parts::AllParts;
use card_faces::CardFaces;
use colors::Colors;
use image_uris::ImageUris;
use legalities::Legalities;
use prices::Prices;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::card::models::scryfall_data::rarity::Rarity;

// ======
//  main
// ======

/// Complete card data from Scryfall API.
///
/// This struct mirrors the Scryfall card object schema and is used for both
/// create and get operations. Fields are organized into three categories:
/// - **Core fields**: Card identification and URIs
/// - **Gameplay fields**: Rules-relevant properties (CMC, colors, oracle text)
/// - **Print fields**: Printing-specific properties (artist, set, rarity)
///
/// # Design Note
///
/// This struct derives `sqlx::FromRow` (when `zerver` feature is enabled) to avoid
/// maintaining separate domain and database models for this large structure.
/// If the database ever changes, only the derive macro needs replacement.
// =]:^{O
#[cfg_attr(feature = "zerver", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ScryfallData {
    // ==================
    // Core Card Fields
    // ==================
    // Basic card identification and platform-specific IDs

    /// This card's Arena ID, if any. A large percentage of cards are not available on Arena and do not have this ID.
    pub arena_id: Option<i32>,

    /// A unique ID for this card in Scryfall's database.
    pub id: uuid::Uuid,

    /// A language code for this printing.
    pub lang: String,

    /// This card's Magic Online ID (also known as the Catalog ID), if any.
    pub mtgo_id: Option<i32>,

    /// This card's foil Magic Online ID (also known as the Catalog ID), if any.
    pub mtgo_foil_id: Option<i32>,

    /// This card's multiverse IDs on Gatherer, if any, as an array of integers.
    pub multiverse_ids: Option<Vec<i32>>,

    /// This card's ID on TCGplayer's API, also known as the productId.
    pub tcgplayer_id: Option<i32>,

    /// This card's ID on TCGplayer's API, for its etched version if separate.
    pub tcgplayer_etched_id: Option<i32>,

    /// This card's ID on Cardmarket's API, also known as the idProduct.
    pub cardmarket_id: Option<i32>,

    /// A content type for this object, always "card".
    pub object: String,

    /// A code for this card's layout. Common values: "normal", "split", "transform", "modal_dfc", "meld", "leveler", "adventure".
    pub layout: String,

    /// A unique ID for this card's oracle identity consistent across reprints.
    /// Shared by all printings of the same card.
    pub oracle_id: Option<Uuid>,

    /// A link to paginate all reprints for this card on Scryfall's API.
    pub prints_search_uri: String,

    /// A link to this card's rulings list on Scryfall's API.
    pub rulings_uri: String,

    /// A link to this card's permapage on Scryfall's website.
    pub scryfall_uri: String,

    /// A link to this card object on Scryfall's API.
    pub uri: String,

    // ==================
    // Gameplay Fields
    // ==================
    // Properties relevant to game rules and deck building

    /// If this card relates to other cards, an array with Related Card Objects.
    pub all_parts: Option<AllParts>,

    /// An array of Card Face objects, if this card is multifaced.
    pub card_faces: Option<CardFaces>,

    /// The card's mana value (converted mana cost). Some funny cards have fractional mana costs.
    pub cmc: Option<f64>,

    /// This card's color identity.
    pub color_identity: Colors,

    /// The colors in this card's color indicator, if any.
    pub color_indicator: Option<Colors>,

    /// This card's colors, if the overall card has colors defined by rules.
    pub colors: Option<Colors>,

    /// This face's defense, if any.
    pub defense: Option<String>,

    /// This card's overall rank/popularity on EDHREC. Lower values indicate more popular.
    pub edhrec_rank: Option<i32>,

    /// True if this card is on the Commander Game Changer list.
    pub game_changer: Option<bool>,

    /// This card's hand modifier, if it is a Vanguard card.
    pub hand_modifier: Option<String>,

    /// An array of keywords that this card uses.
    pub keywords: Option<Vec<String>>,

    /// An object describing the legality of this card across play formats.
    pub legalities: Legalities,

    /// This card's life modifier, if it is a Vanguard card.
    pub life_modifier: Option<String>,

    /// This loyalty if any. Some cards have non-numeric loyalties.
    pub loyalty: Option<String>,

    /// The mana cost for this card in Scryfall notation (e.g., "{2}{R}{R}"). Empty string if absent.
    pub mana_cost: Option<String>,

    /// The name of this card. Multiple faces separated by ␣//␣.
    pub name: String,

    /// The Oracle text for this card, if any.
    pub oracle_text: Option<String>,

    /// This card's rank/popularity on Penny Dreadful.
    pub penny_rank: Option<i32>,

    /// This card's power, if any. Some powers are non-numeric.
    pub power: Option<String>,

    /// Colors of mana that this card could produce.
    pub produced_mana: Option<Vec<String>>,

    /// True if this card is on the Reserved List.
    pub reserved: bool,

    /// This card's toughness, if any. Some are non-numeric.
    pub toughness: Option<String>,

    /// The type line of this card.
    pub type_line: Option<String>,

    // ==================
    // Print Fields
    // ==================
    // Properties unique to this specific printing

    /// The name of the illustrator of this card.
    pub artist: Option<String>,

    /// The IDs of the artists that illustrated this card.
    pub artist_ids: Option<Vec<Uuid>>,

    /// The lit Unfinity attractions lights on this card, if any (e.g., ["1", "3", "6"]).
    #[serde(default, deserialize_with = "deserialize_int_or_string_array")]
    pub attraction_lights: Option<Vec<String>>,

    /// Whether this card is found in boosters.
    pub booster: bool,

    /// This card's border color: black, white, borderless, yellow, silver, or gold.
    pub border_color: String,

    /// The Scryfall ID for the card back design present.
    pub card_back_id: Option<uuid::Uuid>,

    /// This card's collector number. Can contain non-numeric characters.
    pub collector_number: String,

    /// True if you should avoid using this print downstream.
    pub content_warning: Option<bool>,

    /// True if this card was only released in a video game.
    pub digital: bool,

    /// Flags indicating if this card comes in foil, nonfoil, or etched finishes.
    pub finishes: Vec<String>,

    /// The just-for-fun name printed on the card.
    pub flavor_name: Option<String>,

    /// The flavor text, if any.
    pub flavor_text: Option<String>,

    /// This card's frame effects, if any.
    pub frame_effects: Option<Vec<String>>,

    /// This card's frame layout.
    pub frame: String,

    /// True if this card's artwork is larger than normal.
    pub full_art: bool,

    /// A list of games this card is available in.
    pub games: Option<Vec<String>>,

    /// True if this card's imagery is high resolution.
    pub highres_image: bool,

    /// A unique identifier for the card artwork.
    pub illustration_id: Option<Uuid>,

    /// A computer-readable indicator for image state.
    pub image_status: String,

    /// An object listing available imagery for this card.
    pub image_uris: Option<ImageUris>,

    /// True if this card is oversized.
    pub oversized: bool,

    /// An object containing daily price information for this card.
    pub prices: Prices,

    /// The localized name printed on this card, if any.
    pub printed_name: Option<String>,

    /// The localized text printed on this card, if any.
    pub printed_text: Option<String>,

    /// The localized type line printed on this card, if any.
    pub printed_type_line: Option<String>,

    /// True if this card is a promotional print.
    pub promo: bool,

    /// An array describing what categories of promo cards this falls into.
    pub promo_types: Option<Vec<String>>,

    /// An object providing URIs to this card's listings on marketplaces.
    pub purchase_uris: Option<serde_json::Value>,

    /// This card's rarity: common, uncommon, rare, special, mythic, or bonus.
    pub rarity: Rarity,

    /// An object providing URIs to this card on Magic resources.
    pub related_uris: serde_json::Value,

    /// The date this card was first released.
    pub released_at: chrono::NaiveDate,

    /// True if this card is a reprint.
    pub reprint: bool,

    /// A link to this card's set on Scryfall's website.
    pub scryfall_set_uri: String,

    /// This card's full set name.
    pub set_name: String,

    /// A link to paginate this card's set on the Scryfall API.
    pub set_search_uri: String,

    /// The type of set this printing is in.
    pub set_type: String,

    /// A link to this card's set object on Scryfall's API.
    pub set_uri: String,

    /// This card's set code.
    pub set: String,

    /// This card's Set object UUID.
    pub set_id: uuid::Uuid,

    /// True if this card is a Story Spotlight.
    pub story_spotlight: bool,

    /// True if the card is printed without text.
    pub textless: bool,

    /// Whether this card is a variation of another printing.
    pub variation: bool,

    /// The printing ID of the printing this card is a variation of.
    pub variation_of: Option<Uuid>,

    /// The security stamp on this card, if any.
    pub security_stamp: Option<String>,

    /// This card's watermark, if any.
    pub watermark: Option<String>,

    /// The date this card was previewed.
    pub preview_previewed_at: Option<chrono::NaiveDate>,

    /// A link to the preview for this card.
    pub preview_source_uri: Option<String>,

    /// The name of the source that previewed this card.
    pub preview_source: Option<String>,
}

/// for deserializing `INT[]`, `TEXT[]` or `VARCHAR[]` into `Vec<String>`
///
/// used on a single field in ScryfallData
fn deserialize_int_or_string_array<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    match value {
        Some(Value::Array(arr)) => Ok(Some(arr.into_iter().map(|v| v.to_string()).collect())),
        _ => Ok(None),
    }
}
