//! Domain values for unit tests. zwipe-core's own builders sit behind its
//! `cfg(test)`, so this crate deserializes a fixture instead.

use uuid::Uuid;
use zwipe_core::domain::deck::DeckEntry;
use zwipe_core::domain::deck::quantity::Quantity;

/// A deck entry for `name` holding `quantity` copies on the main deck, with
/// fresh card ids so entries never collide.
pub(crate) fn deck_entry(name: &str, quantity: i32) -> DeckEntry {
    let mut entry: DeckEntry = serde_json::from_str(include_str!("test_fixtures/deck_entry.json"))
        .expect("the fixture matches DeckEntry");
    let id = Uuid::new_v4();
    entry.card.card_profile.scryfall_data_id = id;
    entry.card.scryfall_data.id = id;
    entry.deck_card.scryfall_data_id = id;
    name.clone_into(&mut entry.card.scryfall_data.name);
    entry.deck_card.quantity = Quantity::new(quantity).expect("a valid test quantity");
    entry
}
