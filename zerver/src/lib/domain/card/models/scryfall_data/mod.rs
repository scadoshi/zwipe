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
    /// Magic: The Gathering Arena ID. `None` if not available on Arena.
    pub arena_id: Option<i32>,

    /// Scryfall UUID for this specific card printing (unique per print).
    pub id: uuid::Uuid,

    /// Language code (e.g., "en", "ja", "de"). See Scryfall language codes.
    pub lang: String,

    /// Magic: The Gathering Online (MTGO) ID for regular version. `None` if not on MTGO.
    pub mtgo_id: Option<i32>,

    /// MTGO ID for foil version. `None` if no foil version exists on MTGO.
    pub mtgo_foil_id: Option<i32>,

    /// Multiverse IDs from Gatherer. A card may have multiple IDs. `None` if not on Gatherer.
    pub multiverse_ids: Option<Vec<i32>>,

    /// TCGplayer product ID for regular version. `None` if not available.
    pub tcgplayer_id: Option<i32>,

    /// TCGplayer product ID for etched foil version. `None` if no etched version.
    pub tcgplayer_etched_id: Option<i32>,

    /// Cardmarket (European marketplace) product ID. `None` if not available.
    pub cardmarket_id: Option<i32>,

    /// Scryfall object type. Always "card" for card objects.
    pub object: String,

    /// Card layout type. Common values: "normal", "split", "transform", "modal_dfc", "meld", "leveler", "adventure".
    pub layout: String,

    /// Oracle ID - shared by all printings of the same card (e.g., all "Lightning Bolt" printings share one `oracle_id`).
    /// `None` for tokens and special cards without oracle text.
    pub oracle_id: Option<Uuid>,

    /// Scryfall API URI to search for all other printings of this card.
    pub prints_search_uri: String,

    /// Scryfall API URI to fetch rulings for this card.
    pub rulings_uri: String,

    /// Human-readable Scryfall web page URL for this card.
    pub scryfall_uri: String,

    /// Scryfall API URI for this specific card object.
    pub uri: String,

    // ==================
    // Gameplay Fields
    // ==================
    // Properties relevant to game rules and deck building
    /// Related card parts for split/meld/transform cards. `None` for single-faced cards.
    pub all_parts: Option<AllParts>,

    /// Card faces for double-faced/transform/modal cards. `None` for single-faced cards.
    pub card_faces: Option<CardFaces>,

    /// Converted mana cost (CMC). `None` for cards without mana costs (lands, tokens).
    pub cmc: Option<f64>,

    /// Color identity for Commander format (includes mana symbols in text). Always present.
    pub color_identity: Colors,

    /// Color indicator (colored dot) for cards like Pact of Negation. `None` if no indicator.
    pub color_indicator: Option<Colors>,

    /// Card colors based on mana cost and color indicator. `None` for colorless cards.
    pub colors: Option<Colors>,

    /// Defense value for battle cards (e.g., "3"). `None` for non-battle cards.
    pub defense: Option<String>,

    /// EDHREC Commander format popularity ranking (lower = more popular). `None` if not ranked.
    pub edhrec_rank: Option<i32>,

    /// Unclear - internal Scryfall flag. Rarely populated.
    pub game_changer: Option<bool>,

    /// Starting hand size modifier for Vanguard cards (e.g., "+1", "-2"). `None` for non-Vanguard.
    pub hand_modifier: Option<String>,

    /// Keyword abilities (e.g., "Flying", "Haste"). `None` if no keywords.
    pub keywords: Option<Vec<String>>,

    /// Format legality status (Standard, Modern, Commander, etc.). Always present.
    pub legalities: Legalities,

    /// Starting life modifier for Vanguard cards (e.g., "+5", "-3"). `None` for non-Vanguard.
    pub life_modifier: Option<String>,

    /// Loyalty value for planeswalkers (e.g., "3"). `None` for non-planeswalkers.
    pub loyalty: Option<String>,

    /// Mana cost in Scryfall notation (e.g., "{2}{R}{R}"). `None` for lands and tokens.
    pub mana_cost: Option<String>,

    /// Card name. Always present.
    pub name: String,

    /// Oracle text (official rules text). `None` for cards without text (vanilla creatures, lands).
    pub oracle_text: Option<String>,

    /// Penny Dreadful format popularity ranking. `None` if not legal in Penny Dreadful.
    pub penny_rank: Option<i32>,

    /// Power value for creatures (e.g., "3", "*", "1+*"). `None` for non-creatures.
    pub power: Option<String>,

    /// Mana colors this card can produce (e.g., lands). `None` if doesn't produce mana.
    pub produced_mana: Option<Vec<String>>,

    /// Whether card is on the Reserved List (WotC will never reprint). Always present.
    pub reserved: bool,

    /// Toughness value for creatures (e.g., "3", "*", "2+*"). `None` for non-creatures.
    pub toughness: Option<String>,

    /// Type line (e.g., "Legendary Creature — Human Wizard"). `None` is rare.
    pub type_line: Option<String>,

    // ==================
    // Print Fields
    // ==================
    // Properties unique to this specific printing
    /// Artist name. `None` for tokens or promotional cards without attribution.
    pub artist: Option<String>,

    /// Scryfall UUIDs for all contributing artists. `None` if artist not in Scryfall database.
    pub artist_ids: Option<Vec<Uuid>>,

    /// Attraction lights for Unfinity attraction cards (e.g., ["1", "3", "6"]). `None` for non-attractions.
    #[serde(default, deserialize_with = "deserialize_int_or_string_array")]
    pub attraction_lights: Option<Vec<String>>,

    /// Whether card is available in MTG Arena/MTGO boosters. Always present.
    pub booster: bool,

    /// Border color: "black", "white", "borderless", "silver", "gold". Always present.
    pub border_color: String,

    /// UUID for the card back design. `None` for standard Magic back.
    pub card_back_id: Option<uuid::Uuid>,

    /// Collector number within the set (e.g., "24", "42b", "★15"). Always present.
    pub collector_number: String,

    /// Whether card has sensitive/mature content warning. `None` if no warning.
    pub content_warning: Option<bool>,

    /// Whether card is digital-only (not available in paper). Always present.
    pub digital: bool,

    /// Available finishes (e.g., ["nonfoil", "foil", "etched"]). Always present.
    pub finishes: Vec<String>,

    /// Special frame name for Godzilla series, etc. `None` for standard frame.
    pub flavor_name: Option<String>,

    /// Flavor text (italicized lore text). `None` if no flavor text.
    pub flavor_text: Option<String>,

    /// Special frame effects (e.g., ["legendary", "nyxtouched", "showcase"]). `None` if standard frame.
    pub frame_effects: Option<Vec<String>>,

    /// Frame version: "1993", "1997", "2003", "2015", "future". Always present.
    pub frame: String,

    /// Whether card has full-art frame. Always present.
    pub full_art: bool,

    /// Games this print is available in (e.g., ["paper", "arena", "mtgo"]). `None` is rare.
    pub games: Option<Vec<String>>,

    /// Whether high-resolution scans are available. Always present.
    pub highres_image: bool,

    /// Scryfall UUID for the illustration (shared by alternate arts). `None` if not cataloged.
    pub illustration_id: Option<Uuid>,

    /// Image availability: "missing", "placeholder", "lowres", "highres_scan". Always present.
    pub image_status: String,

    /// Image URIs at various resolutions. `None` for multi-faced cards (see `card_faces`).
    pub image_uris: Option<ImageUris>,

    /// Whether card is oversized (e.g., planechase planes). Always present.
    pub oversized: bool,

    /// Current market prices (USD, EUR, TIX). Always present.
    pub prices: Prices,

    /// Card name in printed language (for non-English cards). `None` if English or not translated.
    pub printed_name: Option<String>,

    /// Oracle text in printed language. `None` if English or not translated.
    pub printed_text: Option<String>,

    /// Type line in printed language. `None` if English or not translated.
    pub printed_type_line: Option<String>,

    /// Whether card is promotional. Always present.
    pub promo: bool,

    /// Promotional types (e.g., ["prerelease", "bundle"]). `None` if not promotional.
    pub promo_types: Option<Vec<String>>,

    /// Purchase links to retailers (TCGplayer, Cardmarket, etc.). `None` if not available.
    pub purchase_uris: Option<serde_json::Value>,

    /// Card rarity: Common, Uncommon, Rare, Mythic. Always present.
    pub rarity: Rarity,

    /// Related links (EDHREC, Gatherer, etc.). Always present.
    pub related_uris: serde_json::Value,

    /// Official release date for this set. Always present.
    pub released_at: chrono::NaiveDate,

    /// Whether this is a reprint (false for first printing). Always present.
    pub reprint: bool,

    /// Scryfall set page URI. Always present.
    pub scryfall_set_uri: String,

    /// Full set name (e.g., "Innistrad: Midnight Hunt"). Always present.
    pub set_name: String,

    /// Scryfall API URI to search cards in this set. Always present.
    pub set_search_uri: String,

    /// Set type: "core", "expansion", "masters", "draft_innovation", "commander", etc. Always present.
    pub set_type: String,

    /// Scryfall set API URI. Always present.
    pub set_uri: String,

    /// Set code (e.g., "MID", "NEO", "2XM"). Always present.
    pub set: String,

    /// Scryfall UUID for the set. Always present.
    pub set_id: uuid::Uuid,

    /// Whether card is a Story Spotlight card. Always present.
    pub story_spotlight: bool,

    /// Whether card has no text box. Always present.
    pub textless: bool,

    /// Whether this is an alternate art/frame variation. Always present.
    pub variation: bool,

    /// Scryfall ID of the parent card if this is a variation. `None` if not a variation.
    pub variation_of: Option<Uuid>,

    /// Security stamp type: "oval", "triangle", "acorn", "arena". `None` if no stamp.
    pub security_stamp: Option<String>,

    /// Watermark name (e.g., "set", "colorpie", "wotc"). `None` if no watermark.
    pub watermark: Option<String>,

    /// Preview release date if card was previewed early. `None` if not previewed.
    pub preview_previewed_at: Option<chrono::NaiveDate>,

    /// URI to preview source article. `None` if not previewed.
    pub preview_source_uri: Option<String>,

    /// Name of preview source (website/streamer). `None` if not previewed.
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
