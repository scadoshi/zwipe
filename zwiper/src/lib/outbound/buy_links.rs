//! URL construction for external card retailer bulk-buy tools.

use zwipe_core::domain::deck::DeckEntry;

/// Returns the front face name for double-faced cards, or the full name otherwise.
fn front_face(name: &str) -> &str {
    name.split(" // ").next().unwrap_or(name)
}

/// URL-encodes a card name for buy link URLs.
fn encode_name(name: &str) -> String {
    front_face(name)
        .replace(' ', "+")
        .replace(',', "%2C")
        .replace('\'', "%27")
}

/// Builds a TCGplayer mass-entry URL for the given deck entries.
///
/// `command_zone_names` are cards stored on the deck profile (commander, partner, etc.)
/// that are not part of the entries array. Each is added with quantity 1.
///
/// Format: `https://www.tcgplayer.com/massentry?c=1+Sol+Ring||4+Lightning+Bolt`
pub fn tcgplayer_url(entries: &[DeckEntry], command_zone_names: &[&str]) -> String {
    let mut cards: Vec<String> = command_zone_names
        .iter()
        .map(|name| format!("1+{}", encode_name(name)))
        .collect();

    cards.extend(entries.iter().map(|e| {
        format!(
            "{}+{}",
            *e.deck_card.quantity,
            encode_name(&e.card.scryfall_data.name)
        )
    }));

    format!("https://www.tcgplayer.com/massentry?c={}", cards.join("||"))
}

/// Builds a CardKingdom builder URL for the given deck entries.
///
/// `command_zone_names` are cards stored on the deck profile (commander, partner, etc.)
/// that are not part of the entries array. Each is added with quantity 1.
///
/// Format: `https://www.cardkingdom.com/builder?c=1+Sol+Ring%0A4+Lightning+Bolt`
pub fn cardkingdom_url(entries: &[DeckEntry], command_zone_names: &[&str]) -> String {
    let mut cards: Vec<String> = command_zone_names
        .iter()
        .map(|name| format!("1+{}", encode_name(name)))
        .collect();

    cards.extend(entries.iter().map(|e| {
        format!(
            "{}+{}",
            *e.deck_card.quantity,
            encode_name(&e.card.scryfall_data.name)
        )
    }));

    format!(
        "https://www.cardkingdom.com/builder?c={}",
        cards.join("%0A")
    )
}
