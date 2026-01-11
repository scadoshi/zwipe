pub mod all_parts;
pub mod card_faces;
pub mod colors;
pub mod get_scryfall_data;
pub mod image_uris;
pub mod language;
pub mod legalities;
pub mod prices;
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

/// card data from scryfall
/// used for create and get requests
// =]:^{O
#[cfg_attr(feature = "zerver", derive(sqlx::FromRow))]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
// qualifying usage of FromRow (a sqlx derive macro)
// in my domain logic (where database logic is usually banned)
//
// for now i don't want to have to build this giant structure multiple times
// more maintenance for the little gain that separating these two would have
// if i ever have to switch database, i can just replace the derive macro
pub struct ScryfallData {
    // core card fields
    // cards have the following core properties
    pub arena_id: Option<i32>,
    pub id: uuid::Uuid,
    pub lang: String,
    pub mtgo_id: Option<i32>,
    pub mtgo_foil_id: Option<i32>,
    pub multiverse_ids: Option<Vec<i32>>,
    pub tcgplayer_id: Option<i32>,
    pub tcgplayer_etched_id: Option<i32>,
    pub cardmarket_id: Option<i32>,
    pub object: String,
    pub layout: String,
    pub oracle_id: Option<Uuid>,
    pub prints_search_uri: String,
    pub rulings_uri: String,
    pub scryfall_uri: String,
    pub uri: String,

    // gameplay fields
    // cards have the following properties relevant to the game rules
    pub all_parts: Option<AllParts>,
    pub card_faces: Option<CardFaces>,
    pub cmc: Option<f64>,
    pub color_identity: Colors,
    pub color_indicator: Option<Colors>,
    pub colors: Option<Colors>,
    pub defense: Option<String>,
    pub edhrec_rank: Option<i32>,
    pub game_changer: Option<bool>,
    pub hand_modifier: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub legalities: Legalities,
    pub life_modifier: Option<String>,
    pub loyalty: Option<String>,
    pub mana_cost: Option<String>,
    pub name: String,
    pub oracle_text: Option<String>,
    pub penny_rank: Option<i32>,
    pub power: Option<String>,
    pub produced_mana: Option<Vec<String>>,
    pub reserved: bool,
    pub toughness: Option<String>,
    pub type_line: Option<String>,

    // print fields
    // cards have the following properties unique to their particular re/print
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<Uuid>>,
    #[serde(default, deserialize_with = "deserialize_int_or_string_array")]
    pub attraction_lights: Option<Vec<String>>,
    pub booster: bool,
    pub border_color: String,
    pub card_back_id: Option<uuid::Uuid>,
    pub collector_number: String,
    pub content_warning: Option<bool>,
    pub digital: bool,
    pub finishes: Vec<String>,
    pub flavor_name: Option<String>,
    pub flavor_text: Option<String>,
    pub frame_effects: Option<Vec<String>>,
    pub frame: String,
    pub full_art: bool,
    pub games: Option<Vec<String>>,
    pub highres_image: bool,
    pub illustration_id: Option<Uuid>,
    pub image_status: String,
    pub image_uris: Option<ImageUris>,
    pub oversized: bool,
    pub prices: Prices,
    pub printed_name: Option<String>,
    pub printed_text: Option<String>,
    pub printed_type_line: Option<String>,
    pub promo: bool,
    pub promo_types: Option<Vec<String>>,
    pub purchase_uris: Option<serde_json::Value>,
    pub rarity: Rarity,
    pub related_uris: serde_json::Value,
    pub released_at: chrono::NaiveDate,
    pub reprint: bool,
    pub scryfall_set_uri: String,
    pub set_name: String,
    pub set_search_uri: String,
    pub set_type: String,
    pub set_uri: String,
    pub set: String,
    pub set_id: uuid::Uuid,
    pub story_spotlight: bool,
    pub textless: bool,
    pub variation: bool,
    pub variation_of: Option<Uuid>,
    pub security_stamp: Option<String>,
    pub watermark: Option<String>,
    pub preview_previewed_at: Option<chrono::NaiveDate>,
    pub preview_source_uri: Option<String>,
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
