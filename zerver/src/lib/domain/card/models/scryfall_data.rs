pub mod all_parts;
pub mod card_faces;
pub mod colors;
pub mod image_uris;
pub mod legalities;
pub mod prices;

use all_parts::AllParts;
use card_faces::CardFaces;
use colors::Colors;
use image_uris::ImageUris;
use legalities::Legalities;
use prices::Prices;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
// =]:^{O
use sqlx::FromRow;
use thiserror::Error;

use crate::domain::{card::models::card_profile::CardProfile, DatabaseError};

// =======
//  error
// =======

#[derive(Debug, Error)]
pub enum GetMultipleScryfallDataRequestError {
    #[error("invalid id: {0}")]
    InvalidUuid(uuid::Error),
    #[error("no ids provided")]
    MissingIds,
}

impl From<uuid::Error> for GetMultipleScryfallDataRequestError {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidUuid(value)
    }
}

/// for errors encountered while creating cards
#[derive(Debug, Error)]
pub enum CreateCardError {
    #[error("id already exists")]
    UniqueConstraintViolation(anyhow::Error),
    #[error(transparent)]
    Database(DatabaseError),
    #[error("scryfall data inserted but database returned invalid object: {0}")]
    InvalidScryfallDataFromDatabase(anyhow::Error),
    #[error("card profile created but database returned invalid object: {0}")]
    InvalidCardProfileFromDatabase(anyhow::Error),
}

/// for errors encountered while getting scryfall data
#[derive(Debug, Error)]
pub enum GetScryfallDataError {
    #[error("scryfall data not found")]
    NotFound,
    #[error(transparent)]
    Database(DatabaseError),
    #[error("scryfall data found but database returned invalid object: {0}")]
    InvalidScryfallDataFromDatabase(anyhow::Error),
}

/// for errors encountered while searching cards
/// - NotFound is not a possible enumeration of this
/// because a search request should just return an empty vec
#[derive(Debug, Error)]
pub enum SearchCardError {
    #[error(transparent)]
    Database(DatabaseError),
    #[error("card profile found but database returned invalid object: {0}")]
    InvalidCardProfileFromDatabase(anyhow::Error),
    #[error("scryfall data found but database returned invalid object: {0}")]
    InvalidScryfallDataFromDatabase(anyhow::Error),
}

// ==========
//  requests
// ==========

#[derive(Debug)]
pub struct GetScryfallDataRequest(Uuid);

impl GetScryfallDataRequest {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::try_parse(id)?))
    }

    pub fn id(&self) -> &Uuid {
        &self.0
    }
}

impl<'de> Deserialize<'de> for GetScryfallDataRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = String::deserialize(deserializer).map_err(|e| {
            serde::de::Error::custom(format!(
                "failed to deserialize into string: {}",
                e.to_string()
            ))
        })?;
        GetScryfallDataRequest::new(&id)
            .map_err(|e| serde::de::Error::custom(format!("invalid uuid: {}", e.to_string())))
    }
}

pub struct GetMultipleScryfallDataRequest(Vec<Uuid>);

impl GetMultipleScryfallDataRequest {
    pub fn new(ids: Vec<&str>) -> Result<Self, GetMultipleScryfallDataRequestError> {
        if ids.is_empty() {
            return Err(GetMultipleScryfallDataRequestError::MissingIds);
        }
        Ok(Self(
            ids.into_iter()
                .map(|x| Uuid::try_parse(x))
                .collect::<Result<Vec<Uuid>, uuid::Error>>()?,
        ))
    }

    pub fn ids(&self) -> &Vec<Uuid> {
        &self.0
    }
}

impl From<Vec<CardProfile>> for GetMultipleScryfallDataRequest {
    fn from(value: Vec<CardProfile>) -> Self {
        let ids: Vec<Uuid> = value.into_iter().map(|x| x.scryfall_data_id).collect();
        Self(ids)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCardRequest {
    pub name: Option<String>,
    pub type_line: Option<String>,
    pub set: Option<String>,
    pub rarity: Option<String>,
    pub cmc: Option<f64>,
    pub color_identity: Option<Vec<String>>,
    pub oracle_text: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for SearchCardRequest {
    fn default() -> Self {
        Self {
            name: None,
            type_line: None,
            set: None,
            rarity: None,
            cmc: None,
            color_identity: None,
            oracle_text: None,
            limit: Some(20), // default page size
            offset: Some(0), // start at beginning
        }
    }
}

impl SearchCardRequest {
    pub fn new(
        name: Option<String>,
        type_line: Option<String>,
        set: Option<String>,
        rarity: Option<String>,
        cmc: Option<f64>,
        color_identity: Option<Vec<String>>,
        oracle_text: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Self {
        Self {
            name,
            type_line,
            set,
            rarity,
            cmc,
            color_identity,
            oracle_text,
            limit,
            offset,
        }
    }

    pub fn has_filters(&self) -> bool {
        self.name.is_some()
            || self.type_line.is_some()
            || self.set.is_some()
            || self.rarity.is_some()
            || self.cmc.is_some()
            || self.color_identity.is_some()
            || self.oracle_text.is_some()
            || self.limit.is_some()
            || self.offset.is_some()
    }
}

// ======
//  main
// ======

/// card data from scryfall
/// used for create and get requests
#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq)]
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
    pub cmc: f64,
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
    pub type_line: String,

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
    pub rarity: String,
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
