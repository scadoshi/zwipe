use crate::domain::card::models::scryfall_data::ScryfallData;
use sqlx::Postgres;
use sqlx::QueryBuilder;

/// every `ScryfallData` field line separated for various uses
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

/// comma separates non-empty lines in `SCRYFALL_DATA_FIELDS`
pub fn scryfall_data_fields() -> String {
    SCRYFALL_DATA_FIELDS
        .trim()
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<String>>()
        .join(",")
}

/// counts the number of non-empty lines in `SCRYFALL_DATA_FIELDS`
pub fn scryfall_data_field_count() -> usize {
    SCRYFALL_DATA_FIELDS
        .trim()
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .count()
}

/// prepares `SCRYFALL_DATA_FIELDS` for an `ON CONFLICT` clause for use while upserting in bulk
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

/// binds all `SCRYFALL_DATA_FIELDS` onto a `QueryBuilder` with given card's data
pub trait BindScryfallDataFields {
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
        self.push_bind(card.rarity.clone());
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

/// binds many cards onto a `QueryBuilder` using the above
pub trait BindCards {
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
