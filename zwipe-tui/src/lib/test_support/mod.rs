//! Builders for tests, starting from one real card served by the API.

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{Board, Deck, DeckCard, DeckEntry, DeckProfile, Quantity};

const CARD_JSON: &str = include_str!("card.json");

/// A real card renamed to `name`, with fresh printing and oracle IDs, no faces, and no cost, rules, stats or flavor text.
pub(crate) fn card(name: &str) -> Card {
    let mut card: Card = serde_json::from_str(CARD_JSON).unwrap();
    let id = Uuid::new_v4();
    card.card_profile.scryfall_data_id = id;
    let data = &mut card.scryfall_data;
    data.id = id;
    data.oracle_id = Some(Uuid::new_v4());
    data.name = name.to_owned();
    data.card_faces = None;
    data.mana_cost = None;
    data.type_line = None;
    data.oracle_text = None;
    data.power = None;
    data.toughness = None;
    data.loyalty = None;
    data.defense = None;
    data.flavor_text = None;
    card
}

/// An empty deck named `name` with no commander and no format.
pub(crate) fn deck_profile(name: &str) -> DeckProfile {
    serde_json::from_value(serde_json::json!({
        "id": Uuid::new_v4(),
        "name": name,
        "commander_id": null,
        "partner_commander_id": null,
        "background_id": null,
        "signature_spell_id": null,
        "format": null,
        "land_target": null,
        "price_target": null,
        "price_target_currency": null,
        "user_id": Uuid::new_v4(),
        "card_count": 0,
        "commander_name": null,
        "partner_commander_name": null,
        "background_name": null,
        "signature_spell_name": null,
    }))
    .unwrap()
}

/// A deck holding one copy of each card on the main board.
pub(crate) fn deck(deck_profile: DeckProfile, cards: Vec<Card>) -> Deck {
    let deck_id = deck_profile.id;
    let entries = cards
        .into_iter()
        .map(|card| DeckEntry {
            deck_card: DeckCard {
                deck_id,
                scryfall_data_id: card.scryfall_data.id,
                oracle_id: card.scryfall_data.oracle_id.unwrap(),
                quantity: Quantity::new(1).unwrap(),
                board: Board::default(),
                mvp_at: None,
            },
            card,
        })
        .collect();
    Deck {
        deck_profile,
        entries,
        warnings: Vec::new(),
        command_zone_cards: Vec::new(),
    }
}
