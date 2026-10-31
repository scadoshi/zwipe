//! Retailer links for buying a whole deck, ported from the app's `buy_links.rs`.

use zwipe_core::domain::deck::{Deck, DeckEntry};

/// A double-faced card's front face, or the whole name.
fn front_face(name: &str) -> &str {
    name.split(" // ").next().unwrap_or(name)
}

/// The front face with spaces as `+` and commas and apostrophes escaped. Nothing else is escaped, as in the app.
pub(crate) fn encode_name(name: &str) -> String {
    front_face(name)
        .replace(' ', "+")
        .replace(',', "%2C")
        .replace('\'', "%27")
}

/// Each command-zone card as `1+{name}`, then each entry as `{qty}+{name}`.
fn items(entries: &[&DeckEntry], command_zone_names: &[&str]) -> Vec<String> {
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
    cards
}

/// The `TCGplayer` mass-entry link.
pub(crate) fn tcgplayer_url(entries: &[&DeckEntry], command_zone_names: &[&str]) -> String {
    format!(
        "https://www.tcgplayer.com/massentry?c={}",
        items(entries, command_zone_names).join("||")
    )
}

/// The `CardKingdom` builder link.
pub(crate) fn cardkingdom_url(entries: &[&DeckEntry], command_zone_names: &[&str]) -> String {
    format!(
        "https://www.cardkingdom.com/builder?c={}",
        items(entries, command_zone_names).join("%0A")
    )
}

/// The deck's `TCGplayer` and `CardKingdom` links: the command zone, then the main and side boards in served order. The maybeboard stays out.
pub(crate) fn links(deck: &Deck) -> (String, String) {
    let profile = &deck.deck_profile;
    let names: Vec<&str> = [
        profile.commander_name.as_deref(),
        profile.partner_commander_name.as_deref(),
        profile.background_name.as_deref(),
        profile.signature_spell_name.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect();
    let entries: Vec<&DeckEntry> = deck
        .entries
        .iter()
        .filter(|e| e.deck_card.board.is_active() || e.deck_card.board.is_sideboard())
        .collect();
    (
        tcgplayer_url(&entries, &names),
        cardkingdom_url(&entries, &names),
    )
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::deck::{Board, Quantity};

    use super::*;
    use crate::test_support::{card, deck, deck_profile};

    #[test]
    fn names_keep_the_front_face_and_escape_three_characters() {
        assert_eq!(
            encode_name("Delver of Secrets // Insectile Aberration"),
            "Delver+of+Secrets"
        );
        assert_eq!(
            encode_name("Urza, Lord High Artificer"),
            "Urza%2C+Lord+High+Artificer"
        );
        assert_eq!(encode_name("Gaea's Cradle"), "Gaea%27s+Cradle");
    }

    #[test]
    fn links_put_the_command_zone_first_and_skip_the_maybeboard() {
        let mut profile = deck_profile("Krenko");
        profile.commander_name = Some("Krenko, Mob Boss".to_owned());
        let mut built = deck(
            profile,
            vec![card("Sol Ring"), card("Lightning Bolt"), card("Opt")],
        );
        built.entries[1].deck_card.quantity = Quantity::new(4).unwrap();
        built.entries[1].deck_card.board = Board::Sideboard;
        built.entries[2].deck_card.board = Board::Maybeboard;
        let (tcg, ck) = links(&built);
        assert_eq!(
            tcg,
            "https://www.tcgplayer.com/massentry?c=1+Krenko%2C+Mob+Boss||1+Sol+Ring||4+Lightning+Bolt"
        );
        assert_eq!(
            ck,
            "https://www.cardkingdom.com/builder?c=1+Krenko%2C+Mob+Boss%0A1+Sol+Ring%0A4+Lightning+Bolt"
        );
    }
}
