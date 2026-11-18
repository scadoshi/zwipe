//! Deck rules: what validation warns about, for any deck it is handed.

use std::collections::BTreeSet;

use proptest::prelude::*;
use uuid::Uuid;

use super::strategies::colors;
use crate::domain::card::Card;
use crate::domain::card::scryfall_data::colors::{Color, Colors};
use crate::domain::deck::deck::DeckEntry;
use crate::domain::deck::validate_deck::{DeckCommandZone, validate_deck};
use crate::domain::deck::{DeckProfile, DeckWarning, Format};
use crate::test_utils::{make_card, make_entry};

/// What the color identity warning says, after the card's name.
const OUTSIDE_IDENTITY: &str = "is outside your commander's color identity";

/// An empty deck in `format`.
fn profile(format: Format) -> DeckProfile {
    let mut profile: DeckProfile = serde_json::from_value(serde_json::json!({
        "id": Uuid::new_v4(),
        "name": "properties",
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
    .unwrap();
    profile.format = Some(format);
    profile
}

/// A card named `name` whose color identity is `identity`.
fn card_of(name: &str, identity: &[Color]) -> Card {
    let mut card = make_card(name);
    card.scryfall_data.color_identity = Colors::from(identity.iter().copied());
    card
}

/// A deck card of `name` in `identity`, one copy on the main board.
fn entry_of(name: &str, identity: &[Color]) -> DeckEntry {
    let mut entry = make_entry(name, 1);
    entry.card.scryfall_data.color_identity = Colors::from(identity.iter().copied());
    entry
}

proptest! {
    /// Guards the commander color rule: a main board card is flagged exactly when one of its
    /// colors is outside the commander's, partner's and background's together, and a
    /// command zone card listed in the deck never is.
    #[test]
    fn a_card_is_flagged_exactly_when_its_identity_leaves_the_command_zone_s(
        commander in colors(),
        partner in prop::option::of(colors()),
        background in prop::option::of(colors()),
        cards in prop::collection::vec(colors(), 0..8),
        zone_listed in any::<bool>(),
    ) {
        let commander = card_of("Commander", &commander);
        let partner = partner.map(|p| card_of("Partner", &p));
        let background = background.map(|b| card_of("Background", &b));
        let allowed: BTreeSet<Color> = [Some(&commander), partner.as_ref(), background.as_ref()]
            .into_iter()
            .flatten()
            .flat_map(|c| c.scryfall_data.color_identity.iter().copied())
            .collect();

        let mut entries: Vec<DeckEntry> = cards
            .iter()
            .enumerate()
            .map(|(i, identity)| entry_of(&format!("Card {i}"), identity))
            .collect();
        if zone_listed {
            for zone_card in [Some(&commander), partner.as_ref(), background.as_ref()].into_iter().flatten() {
                let mut entry = make_entry(&zone_card.scryfall_data.name, 1);
                entry.card = zone_card.clone();
                entry.deck_card.scryfall_data_id = zone_card.scryfall_data.id;
                entries.push(entry);
            }
        }
        let zone = DeckCommandZone {
            commander: Some(&commander),
            partner_commander: partner.as_ref(),
            background: background.as_ref(),
            signature_spell: None,
        };

        let flagged: BTreeSet<Uuid> = validate_deck(&profile(Format::Commander), &entries, &zone)
            .iter()
            .filter(|w| w.contains(OUTSIDE_IDENTITY))
            .filter_map(DeckWarning::scryfall_data_id)
            .collect();
        let outside: BTreeSet<Uuid> = entries
            .iter()
            .filter(|e| e.card.scryfall_data.color_identity.iter().any(|c| !allowed.contains(c)))
            .map(|e| e.card.scryfall_data.id)
            .collect();
        prop_assert_eq!(flagged, outside);
    }
}
