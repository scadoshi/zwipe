//! The deck's command zone, read from its profile for the deck page.

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Deck;
use zwipe_ui::deck_cards::Zone;

/// The command zone's cards in their slots, matched by printing id.
pub(crate) fn zone(deck: &Deck) -> Zone {
    let profile = &deck.deck_profile;
    let find = |id: Option<Uuid>| -> Option<Card> {
        let id = id?;
        deck.command_zone_cards
            .iter()
            .find(|c| c.scryfall_data.id == id)
            .cloned()
    };
    Zone {
        commander: find(profile.commander_id),
        partner: find(profile.partner_commander_id),
        background: find(profile.background_id),
        signature: find(profile.signature_spell_id),
    }
}
