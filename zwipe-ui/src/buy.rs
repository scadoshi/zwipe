//! Retailer links for buying a whole deck or one card, ported from the app's `buy_links.rs`.

use zwipe_core::domain::deck::{Deck, DeckEntry};

/// The store dialog's link to `TCGplayer`.
pub const TCGPLAYER: &str = "TCGplayer \u{2197}";

/// The store dialog's link to `Card Kingdom`.
pub const CARD_KINGDOM: &str = "Card Kingdom \u{2197}";

/// The store dialog's button that closes it, before the retailer links.
pub const CLOSE: &str = "Close";

/// The cart button's name for a screen reader and its tooltip.
pub const CART_LABEL: &str = "Buy";

/// The store dialog's body for a card group's cards, a List band's or a stack's.
pub const GROUP_BODY: &str = "Buy this group's cards from a retailer.";

/// A retailer link in the store dialog.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BuyLink {
    /// The link's text.
    pub label: &'static str,
    /// Where it opens.
    pub url: String,
}

/// The store dialog's links, `TCGplayer` then `Card Kingdom`, to `tcgplayer` and `cardkingdom`.
#[must_use]
pub fn store_links(tcgplayer: String, cardkingdom: String) -> Vec<BuyLink> {
    vec![
        BuyLink {
            label: TCGPLAYER,
            url: tcgplayer,
        },
        BuyLink {
            label: CARD_KINGDOM,
            url: cardkingdom,
        },
    ]
}

/// The store dialog's links for `cards`, each with its copies, in order.
#[must_use]
pub fn cards_links<'a>(cards: impl IntoIterator<Item = (i32, &'a str)>) -> Vec<BuyLink> {
    let items = line(cards);
    store_links(
        format!("https://www.tcgplayer.com/massentry?c={}", items.join("||")),
        format!(
            "https://www.cardkingdom.com/builder?c={}",
            items.join("%0A")
        ),
    )
}

/// The store dialog's title for the card group whose header reads `header`: its name without the
/// count, or "these cards" for a color group, whose header is the count alone.
#[must_use]
pub fn group_title(header: &str) -> String {
    let name = header
        .rsplit_once('(')
        .filter(|(_, count)| count.ends_with(')'))
        .map_or(header, |(name, _)| name)
        .trim();
    if name.is_empty() {
        "Buy these cards".to_owned()
    } else {
        format!("Buy {name}")
    }
}

/// A double-faced card's front face, or the whole name.
fn front_face(name: &str) -> &str {
    name.split(" // ").next().unwrap_or(name)
}

/// The front face with spaces as `+` and commas and apostrophes escaped. Nothing else is escaped, as in the app.
#[must_use]
pub fn encode_name(name: &str) -> String {
    front_face(name)
        .replace(' ', "+")
        .replace(',', "%2C")
        .replace('\'', "%27")
}

/// Each card as `{qty}+{name}`.
fn line<'a>(cards: impl IntoIterator<Item = (i32, &'a str)>) -> Vec<String> {
    cards
        .into_iter()
        .map(|(qty, name)| format!("{qty}+{}", encode_name(name)))
        .collect()
}

/// Each command-zone card as `1+{name}`, then each entry as `{qty}+{name}`.
fn items(entries: &[&DeckEntry], command_zone_names: &[&str]) -> Vec<String> {
    line(
        command_zone_names.iter().map(|name| (1, *name)).chain(
            entries
                .iter()
                .map(|e| (*e.deck_card.quantity, e.card.scryfall_data.name.as_str())),
        ),
    )
}

/// The `TCGplayer` mass-entry link.
#[must_use]
pub fn tcgplayer_url(entries: &[&DeckEntry], command_zone_names: &[&str]) -> String {
    format!(
        "https://www.tcgplayer.com/massentry?c={}",
        items(entries, command_zone_names).join("||")
    )
}

/// The `CardKingdom` builder link.
#[must_use]
pub fn cardkingdom_url(entries: &[&DeckEntry], command_zone_names: &[&str]) -> String {
    format!(
        "https://www.cardkingdom.com/builder?c={}",
        items(entries, command_zone_names).join("%0A")
    )
}

/// One card's `TCGplayer` link: its product page when the card's `TCGplayer` id is known, else a mass entry of one copy by name.
#[must_use]
pub fn tcgplayer_card_url(name: &str, tcgplayer_id: Option<i32>) -> String {
    match tcgplayer_id {
        Some(id) => format!("https://www.tcgplayer.com/product/{id}"),
        None => format!(
            "https://www.tcgplayer.com/massentry?c=1+{}",
            encode_name(name)
        ),
    }
}

/// One card's `CardKingdom` builder link, one copy.
#[must_use]
pub fn cardkingdom_card_url(name: &str) -> String {
    format!(
        "https://www.cardkingdom.com/builder?c=1+{}",
        encode_name(name)
    )
}

/// The deck's `TCGplayer` and `CardKingdom` links: the command zone, then the main and side boards in served order. The maybeboard stays out.
#[must_use]
pub fn links(deck: &Deck) -> (String, String) {
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
    fn a_set_of_cards_links_to_both_stores_with_its_copies() {
        let links = cards_links([
            (2, "Sol Ring"),
            (1, "Delver of Secrets // Insectile Aberration"),
        ]);
        assert_eq!(
            links,
            [
                BuyLink {
                    label: TCGPLAYER,
                    url: "https://www.tcgplayer.com/massentry?c=2+Sol+Ring||1+Delver+of+Secrets"
                        .to_owned(),
                },
                BuyLink {
                    label: CARD_KINGDOM,
                    url: "https://www.cardkingdom.com/builder?c=2+Sol+Ring%0A1+Delver+of+Secrets"
                        .to_owned(),
                },
            ]
        );
    }

    #[test]
    fn a_group_s_dialog_names_it_without_the_count() {
        assert_eq!(group_title("Creatures (12)"), "Buy Creatures");
        assert_eq!(group_title("Commander"), "Buy Commander");
        assert_eq!(
            group_title("(3)"),
            "Buy these cards",
            "a color group shows pips, not a name"
        );
        assert_eq!(store_links("t".to_owned(), "c".to_owned()).len(), 2);
    }

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

    #[test]
    fn single_card_links_encode_the_front_face_only() {
        assert_eq!(
            tcgplayer_card_url("Fire // Ice", None),
            "https://www.tcgplayer.com/massentry?c=1+Fire"
        );
        assert_eq!(
            tcgplayer_card_url("Sol Ring", Some(42)),
            "https://www.tcgplayer.com/product/42"
        );
        assert_eq!(
            cardkingdom_card_url("Urza's Saga, Again"),
            "https://www.cardkingdom.com/builder?c=1+Urza%27s+Saga%2C+Again"
        );
    }
}
