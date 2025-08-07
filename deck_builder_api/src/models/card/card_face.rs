use diesel::sql_types::Uuid;
use serde::{Deserialize, Serialize};

use crate::models::card::image_uris::ImageUris;

/// To be stored against card
/// against the "card_faces" field
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CardFace {
    artist: Option<String>,
    artist_id: Option<Uuid>,
    cmc: Option<f32>,
    color_indicator: Option<Vec<String>>,
    colors: Option<Vec<String>>,
    defense: Option<String>,
    flavor_text: Option<String>,
    illustration_id: Option<Uuid>,
    image_uris: Option<ImageUris>,
    layout: Option<String>,
    loyalty: Option<String>,
    mana_cost: String,
    name: String,
    object: String,
    oracle_id: Option<Uuid>,
    oracle_text: Option<String>,
    power: Option<String>,
    printed_name: Option<String>,
    printed_text: Option<String>,
    printed_type_line: Option<String>,
    toughness: Option<String>,
    type_line: Option<String>,
    watermark: Option<String>,
}
