//! Scryfall data field helpers for SQL query building.
//!
//! Provides utilities for binding all 88 Scryfall card fields to SQLx query builders,
//! enabling efficient bulk insert/update operations.

use crate::domain::card::models::scryfall_data::ScryfallData;
use sqlx::Postgres;
use sqlx::QueryBuilder;

/// Every [`ScryfallData`] field name, line-separated for SQL generation.
///
/// This constant serves as the single source of truth for field ordering,
/// ensuring consistency between INSERT column lists and VALUES bindings.
const SCRYFALL_DATA_FIELDS: &str = r#"
    arena_id
    id
    lang
    mtgo_id
    mtgo_foil_id
    multiverse_ids
    tcgplayer_id
    tcgplayer_etched_id
    cardmarket_id
    object
    layout
    oracle_id
    prints_search_uri
    rulings_uri
    scryfall_uri
    uri
    all_parts
    card_faces
    cmc
    color_identity
    color_indicator
    colors
    defense
    edhrec_rank
    game_changer
    hand_modifier
    keywords
    legalities
    life_modifier
    loyalty
    mana_cost
    name
    oracle_text
    penny_rank
    power
    produced_mana
    reserved
    toughness
    type_line
    artist
    artist_ids
    attraction_lights
    booster
    border_color
    card_back_id
    collector_number
    content_warning
    digital
    finishes
    flavor_name
    flavor_text
    frame_effects
    frame
    full_art
    games
    highres_image
    illustration_id
    image_status
    image_uris
    oversized
    prices
    printed_name
    printed_text
    printed_type_line
    promo
    promo_types
    purchase_uris
    rarity
    related_uris
    released_at
    reprint
    scryfall_set_uri
    set_name
    set_search_uri
    set_type
    set_uri
    set
    set_id
    story_spotlight
    textless
    variation
    variation_of
    security_stamp
    watermark
    preview_previewed_at
    preview_source_uri
    preview_source
"#;

/// Returns a comma-separated list of all Scryfall data field names.
///
/// Used to build the column list for INSERT statements.
pub fn scryfall_data_fields() -> String {
    SCRYFALL_DATA_FIELDS
        .trim()
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<String>>()
        .join(",")
}

/// Returns the count of Scryfall data fields.
///
/// Useful for validating that bindings match the expected column count.
pub fn scryfall_data_field_count() -> usize {
    SCRYFALL_DATA_FIELDS
        .trim()
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .count()
}

/// Generates an `ON CONFLICT` clause for bulk upsert operations.
///
/// Returns SQL that updates all fields when a row with the same `id` already exists.
pub fn bulk_upsert_conflict_fields() -> String {
    " ON CONFLICT (id) DO UPDATE SET ".to_string()
        + SCRYFALL_DATA_FIELDS
            .trim()
            .lines()
            .map(|x| x.trim().to_string())
            .map(|x| x.clone() + " = EXCLUDED." + x.as_str())
            .collect::<Vec<String>>()
            .join(",")
            .as_str()
}

/// Extension trait for binding a single card's fields to a query builder.
///
/// Pushes all 88 Scryfall fields as SQL bind parameters in the correct order,
/// wrapped in parentheses for use in VALUES clauses.
pub trait BindScryfallDataFields {
    /// Binds all fields from the given card to the query builder.
    ///
    /// The fields are wrapped in parentheses: `(field1, field2, ..., fieldN)`.
    fn bind_scryfall_fields(&mut self, card: &ScryfallData) -> &mut Self;
}

impl BindScryfallDataFields for QueryBuilder<'_, Postgres> {
    fn bind_scryfall_fields(&mut self, card: &ScryfallData) -> &mut Self {
        self.push("(");
        // core card fields
        // cards have the following core properties
        self.push_bind(card.arena_id);
        self.push(", ");
        self.push_bind(card.id);
        self.push(", ");
        self.push_bind(card.lang.clone());
        self.push(", ");
        self.push_bind(card.mtgo_id);
        self.push(", ");
        self.push_bind(card.mtgo_foil_id);
        self.push(", ");
        self.push_bind(card.multiverse_ids.clone());
        self.push(", ");
        self.push_bind(card.tcgplayer_id);
        self.push(", ");
        self.push_bind(card.tcgplayer_etched_id);
        self.push(", ");
        self.push_bind(card.cardmarket_id);
        self.push(", ");
        self.push_bind(card.object.clone());
        self.push(", ");
        self.push_bind(card.layout.clone());
        self.push(", ");
        self.push_bind(card.oracle_id);
        self.push(", ");
        self.push_bind(card.prints_search_uri.clone());
        self.push(", ");
        self.push_bind(card.rulings_uri.clone());
        self.push(", ");
        self.push_bind(card.scryfall_uri.clone());
        self.push(", ");
        self.push_bind(card.uri.clone());
        self.push(", ");
        // gameplay fields
        // cards have the following properties relevant to the game rules
        self.push_bind(card.all_parts.clone());
        self.push(", ");
        self.push_bind(card.card_faces.clone());
        self.push(", ");
        self.push_bind(card.cmc);
        self.push(", ");
        self.push_bind(card.color_identity.clone());
        self.push(", ");
        self.push_bind(card.color_indicator.clone());
        self.push(", ");
        self.push_bind(card.colors.clone());
        self.push(", ");
        self.push_bind(card.defense.clone());
        self.push(", ");
        self.push_bind(card.edhrec_rank);
        self.push(", ");
        self.push_bind(card.game_changer);
        self.push(", ");
        self.push_bind(card.hand_modifier.clone());
        self.push(", ");
        self.push_bind(card.keywords.clone());
        self.push(", ");
        self.push_bind(card.legalities.clone());
        self.push(", ");
        self.push_bind(card.life_modifier.clone());
        self.push(", ");
        self.push_bind(card.loyalty.clone());
        self.push(", ");
        self.push_bind(card.mana_cost.clone());
        self.push(", ");
        self.push_bind(card.name.clone());
        self.push(", ");
        self.push_bind(card.oracle_text.clone());
        self.push(", ");
        self.push_bind(card.penny_rank);
        self.push(", ");
        self.push_bind(card.power.clone());
        self.push(", ");
        self.push_bind(card.produced_mana.clone());
        self.push(", ");
        self.push_bind(card.reserved);
        self.push(", ");
        self.push_bind(card.toughness.clone());
        self.push(", ");
        self.push_bind(card.type_line.clone());
        self.push(", ");
        // print fields
        // cards have the following properties unique to their particular re/print
        self.push_bind(card.artist.clone());
        self.push(", ");
        self.push_bind(card.artist_ids.clone());
        self.push(", ");
        self.push_bind(card.attraction_lights.clone());
        self.push(", ");
        self.push_bind(card.booster);
        self.push(", ");
        self.push_bind(card.border_color.clone());
        self.push(", ");
        self.push_bind(card.card_back_id);
        self.push(", ");
        self.push_bind(card.collector_number.clone());
        self.push(", ");
        self.push_bind(card.content_warning);
        self.push(", ");
        self.push_bind(card.digital);
        self.push(", ");
        self.push_bind(card.finishes.clone());
        self.push(", ");
        self.push_bind(card.flavor_name.clone());
        self.push(", ");
        self.push_bind(card.flavor_text.clone());
        self.push(", ");
        self.push_bind(card.frame_effects.clone());
        self.push(", ");
        self.push_bind(card.frame.clone());
        self.push(", ");
        self.push_bind(card.full_art);
        self.push(", ");
        self.push_bind(card.games.clone());
        self.push(", ");
        self.push_bind(card.highres_image);
        self.push(", ");
        self.push_bind(card.illustration_id);
        self.push(", ");
        self.push_bind(card.image_status.clone());
        self.push(", ");
        self.push_bind(card.image_uris.clone());
        self.push(", ");
        self.push_bind(card.oversized);
        self.push(", ");
        self.push_bind(card.prices.clone());
        self.push(", ");
        self.push_bind(card.printed_name.clone());
        self.push(", ");
        self.push_bind(card.printed_text.clone());
        self.push(", ");
        self.push_bind(card.printed_type_line.clone());
        self.push(", ");
        self.push_bind(card.promo);
        self.push(", ");
        self.push_bind(card.promo_types.clone());
        self.push(", ");
        self.push_bind(card.purchase_uris.clone());
        self.push(", ");
        self.push_bind(card.rarity);
        self.push(", ");
        self.push_bind(card.related_uris.clone());
        self.push(", ");
        self.push_bind(card.released_at);
        self.push(", ");
        self.push_bind(card.reprint);
        self.push(", ");
        self.push_bind(card.scryfall_set_uri.clone());
        self.push(", ");
        self.push_bind(card.set_name.clone());
        self.push(", ");
        self.push_bind(card.set_search_uri.clone());
        self.push(", ");
        self.push_bind(card.set_type.clone());
        self.push(", ");
        self.push_bind(card.set_uri.clone());
        self.push(", ");
        self.push_bind(card.set.clone());
        self.push(", ");
        self.push_bind(card.set_id);
        self.push(", ");
        self.push_bind(card.story_spotlight);
        self.push(", ");
        self.push_bind(card.textless);
        self.push(", ");
        self.push_bind(card.variation);
        self.push(", ");
        self.push_bind(card.variation_of);
        self.push(", ");
        self.push_bind(card.security_stamp.clone());
        self.push(", ");
        self.push_bind(card.watermark.clone());
        self.push(", ");
        self.push_bind(card.preview_previewed_at);
        self.push(", ");
        self.push_bind(card.preview_source_uri.clone());
        self.push(", ");
        self.push_bind(card.preview_source.clone());
        self.push(")");
        self
    }
}

/// Extension trait for binding multiple cards to a query builder.
///
/// Produces comma-separated value tuples for bulk INSERT operations.
pub trait BindCards {
    /// Binds all fields from each card, separated by commas.
    ///
    /// Output format: `(card1_fields), (card2_fields), ...`
    fn bind_cards(&mut self, scryfall_data: &[ScryfallData]) -> &mut Self;
}

impl BindCards for QueryBuilder<'_, Postgres> {
    fn bind_cards(&mut self, scryfall_data: &[ScryfallData]) -> &mut Self {
        for (i, card) in scryfall_data.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            self.bind_scryfall_fields(card);
        }
        self
    }
}
