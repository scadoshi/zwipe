//! URL construction for external card retailer bulk-buy tools.

use zwipe::domain::deck::models::deck::DeckEntry;

/// Returns the front face name for double-faced cards, or the full name otherwise.
fn front_face(name: &str) -> &str {
    name.split(" // ").next().unwrap_or(name)
}

/// Builds a TCGplayer mass-entry URL for the given deck entries.
///
/// Format: `https://www.tcgplayer.com/massentry?c=1+Sol+Ring||4+Lightning+Bolt`
pub fn tcgplayer_url(entries: &[DeckEntry]) -> String {
    let cards: Vec<String> = entries
        .iter()
        .map(|e| {
            let name = front_face(&e.card.scryfall_data.name)
                .replace(' ', "+")
                .replace(',', "%2C")
                .replace('\'', "%27");
            format!("{}+{}", *e.deck_card.quantity, name)
        })
        .collect();

    format!(
        "https://www.tcgplayer.com/massentry?c={}",
        cards.join("||")
    )
}

/// Builds a CardKingdom builder URL for the given deck entries.
///
/// Format: `https://www.cardkingdom.com/builder?c=1+Sol+Ring%0A4+Lightning+Bolt`
pub fn cardkingdom_url(entries: &[DeckEntry]) -> String {
    let cards: Vec<String> = entries
        .iter()
        .map(|e| {
            let name = front_face(&e.card.scryfall_data.name)
                .replace(' ', "+")
                .replace(',', "%2C")
                .replace('\'', "%27");
            format!("{}+{}", *e.deck_card.quantity, name)
        })
        .collect();

    format!(
        "https://www.cardkingdom.com/builder?c={}",
        cards.join("%0A")
    )
}
