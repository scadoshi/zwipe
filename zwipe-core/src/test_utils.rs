//! Shared test utilities for constructing domain types.

use crate::domain::{
    card::{
        Card,
        card_profile::CardProfile,
        scryfall_data::{
            ScryfallData, colors::Colors, legalities::Legalities, prices::Prices, rarity::Rarity,
        },
    },
    deck::{Board, DeckCard, deck::DeckEntry, quantity::Quantity},
};
use chrono::NaiveDate;
use uuid::Uuid;

/// Creates a minimal test `DeckEntry` with the given card name and quantity.
pub fn make_entry(name: &str, qty: i32) -> DeckEntry {
    let deck_id = Uuid::new_v4();
    let card = make_card(name);
    let scryfall_data_id = card.scryfall_data.id;
    let oracle_id = card.scryfall_data.oracle_id.unwrap();
    DeckEntry {
        card,
        deck_card: DeckCard {
            deck_id,
            scryfall_data_id,
            oracle_id,
            quantity: Quantity::new(qty).unwrap(),
            board: Board::default(),
            mvp_at: None,
        },
    }
}

/// Creates a minimal test `Card` with sensible defaults.
///
/// Most optional fields are `None`. Mutate the returned card to set
/// specific fields needed for your test.
pub fn make_card(name: &str) -> Card {
    Card {
        card_profile: CardProfile {
            scryfall_data_id: Uuid::new_v4(),
            is_token: false,
            mechanical_categories: vec![],
            oracle_tags: vec![],
            oracle_tags_by_role: Default::default(),
            other_oracle_tags: vec![],
            created_at: NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            updated_at: NaiveDate::from_ymd_opt(2021, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
        },
        scryfall_data: ScryfallData {
            arena_id: None,
            id: Uuid::new_v4(),
            lang: "en".to_string(),
            mtgo_id: None,
            mtgo_foil_id: None,
            multiverse_ids: None,
            tcgplayer_id: None,
            tcgplayer_etched_id: None,
            cardmarket_id: None,
            object: "card".to_string(),
            layout: "normal".to_string(),
            oracle_id: Some(Uuid::new_v4()),
            prints_search_uri: String::new(),
            rulings_uri: String::new(),
            scryfall_uri: String::new(),
            uri: String::new(),
            all_parts: None,
            card_faces: None,
            cmc: None,
            color_identity: Colors::from([]),
            color_indicator: None,
            colors: None,
            defense: None,
            edhrec_rank: None,
            game_changer: None,
            hand_modifier: None,
            keywords: None,
            legalities: Legalities::default(),
            life_modifier: None,
            loyalty: None,
            mana_cost: None,
            name: name.to_string(),
            oracle_text: None,
            penny_rank: None,
            power: None,
            produced_mana: None,
            reserved: false,
            toughness: None,
            type_line: None,
            artist: None,
            artist_ids: None,
            attraction_lights: None,
            booster: false,
            border_color: String::new(),
            card_back_id: None,
            collector_number: String::new(),
            content_warning: None,
            digital: false,
            finishes: vec![],
            flavor_name: None,
            flavor_text: None,
            frame_effects: None,
            frame: String::new(),
            full_art: false,
            games: None,
            highres_image: false,
            illustration_id: None,
            image_status: String::new(),
            image_uris: None,
            oversized: false,
            prices: Prices {
                usd: None,
                usd_foil: None,
                usd_etched: None,
                eur: None,
                eur_foil: None,
                eur_etched: None,
                tix: None,
            },
            printed_name: None,
            printed_text: None,
            printed_type_line: None,
            promo: false,
            promo_types: None,
            purchase_uris: None,
            rarity: Rarity::Common,
            related_uris: serde_json::Value::Null,
            released_at: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            reprint: false,
            scryfall_set_uri: String::new(),
            set_name: String::new(),
            set_search_uri: String::new(),
            set_type: String::new(),
            set_uri: String::new(),
            set: "m21".to_string(),
            set_id: Uuid::new_v4(),
            story_spotlight: false,
            textless: false,
            variation: false,
            variation_of: None,
            security_stamp: None,
            watermark: None,
            preview_previewed_at: None,
            preview_source_uri: None,
            preview_source: None,
        },
    }
}
