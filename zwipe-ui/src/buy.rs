//! Retailer links for buying a whole deck or one card, ported from the app's `buy_links.rs`.

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::domain::deck::{Deck, DeckEntry, deck_metrics};

use crate::deck_cards::keys::{RowInfo, RowKind};

/// The store dialog's link to `TCGplayer`.
pub const TCGPLAYER: &str = "TCGplayer \u{2197}";

/// The store dialog's link to `Card Kingdom`.
pub const CARD_KINGDOM: &str = "Card Kingdom \u{2197}";

/// The store dialog's button that closes it, before the retailer links.
pub const CLOSE: &str = "Close";

/// The cart button's name for a screen reader and its tooltip.
pub const CART_LABEL: &str = "Buy";

/// The store dialog's title for one card.
pub const CARD_TITLE: &str = "Buy card";

/// The store dialog's body for one card.
pub const CARD_BODY: &str = "Buy this card from a retailer.";

/// The store dialog's title for the whole deck.
pub const DECK_TITLE: &str = "Buy deck";

/// The store dialog's body for the whole deck.
pub const DECK_BODY: &str = "Buy this deck's cards from a retailer.";

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

/// The store dialog's links for `cards`, each with its copies, in order: `TCGplayer` gets each
/// card's printing where it has one.
#[must_use]
pub fn cards_links<'a>(cards: impl IntoIterator<Item = StoreCard<'a>>) -> Vec<BuyLink> {
    let cards: Vec<StoreCard<'a>> = cards.into_iter().collect();
    store_links(tcgplayer_mass(&cards), cardkingdom_builder(&cards))
}

/// The store dialog's links for the deck: the command zone, then the main and side boards, as
/// [`links`] builds them.
#[must_use]
pub fn deck_links(deck: &Deck) -> Vec<BuyLink> {
    let (tcgplayer, cardkingdom) = links(deck);
    store_links(tcgplayer, cardkingdom)
}

/// The store dialog's links for one card named `name`: its `TCGplayer` product page when
/// `tcgplayer_id` is known, else a one-copy entry, and a one-copy `Card Kingdom` entry.
#[must_use]
pub fn card_links(name: &str, tcgplayer_id: Option<i32>) -> Vec<BuyLink> {
    store_links(
        tcgplayer_card_url(name, tcgplayer_id),
        cardkingdom_card_url(name),
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

/// A card sent to a store: its copies, its name and its `TCGplayer` product id, which names the
/// exact printing, when known.
pub type StoreCard<'a> = (i32, &'a str, Option<i32>);

/// `TCGplayer`'s mass entry, for Magic, before its `||`-joined lines.
const TCGPLAYER_MASS: &str = "https://www.tcgplayer.com/massentry?productline=Magic&c=";

/// `Card Kingdom`'s builder, before its newline-joined lines.
const CARD_KINGDOM_BUILDER: &str = "https://www.cardkingdom.com/builder?c=";

/// A card's mass-entry line: `{qty}-{product id}` for its exact printing, else `{qty}+{name}`.
fn tcgplayer_line((qty, name, tcgplayer_id): StoreCard<'_>) -> String {
    match tcgplayer_id {
        Some(id) => format!("{qty}-{id}"),
        None => format!("{qty}+{}", encode_name(name)),
    }
}

/// A card's builder line, `{qty}+{name}`: the builder matches by name only, and any set or finish
/// marker leaves the card unrecognized.
fn cardkingdom_line((qty, name, _): StoreCard<'_>) -> String {
    format!("{qty}+{}", encode_name(name))
}

/// The `TCGplayer` mass entry for `cards`.
fn tcgplayer_mass(cards: &[StoreCard<'_>]) -> String {
    let lines: Vec<String> = cards.iter().copied().map(tcgplayer_line).collect();
    format!("{TCGPLAYER_MASS}{}", lines.join("||"))
}

/// The `Card Kingdom` builder for `cards`.
fn cardkingdom_builder(cards: &[StoreCard<'_>]) -> String {
    let lines: Vec<String> = cards.iter().copied().map(cardkingdom_line).collect();
    format!("{CARD_KINGDOM_BUILDER}{}", lines.join("%0A"))
}

/// The deck's cards for a store: each command-zone card once, then each entry with its copies.
fn deck_cards<'a>(
    entries: &[&'a DeckEntry],
    command_zone: &[(&'a str, Option<i32>)],
) -> Vec<StoreCard<'a>> {
    command_zone
        .iter()
        .map(|(name, id)| (1, *name, *id))
        .chain(entries.iter().map(|e| {
            (
                *e.deck_card.quantity,
                e.card.scryfall_data.name.as_str(),
                e.card.scryfall_data.tcgplayer_id,
            )
        }))
        .collect()
}

/// The command zone named on `profile`, in its order (commander, partner, background, signature
/// spell), each with the `TCGplayer` id of its card in `cards` when one of them carries that name.
#[must_use]
pub fn command_zone<'a>(
    profile: &'a DeckProfile,
    cards: &'a [Card],
) -> Vec<(&'a str, Option<i32>)> {
    [
        profile.commander_name.as_deref(),
        profile.partner_commander_name.as_deref(),
        profile.background_name.as_deref(),
        profile.signature_spell_name.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(|name| {
        let id = cards
            .iter()
            .find(|card| card.scryfall_data.name == name)
            .and_then(|card| card.scryfall_data.tcgplayer_id);
        (name, id)
    })
    .collect()
}

/// The `TCGplayer` mass-entry link: `command_zone` (as [`command_zone`] gives it), then `entries`.
#[must_use]
pub fn tcgplayer_url(entries: &[&DeckEntry], command_zone: &[(&str, Option<i32>)]) -> String {
    tcgplayer_mass(&deck_cards(entries, command_zone))
}

/// The `CardKingdom` builder link: `command_zone`, then `entries`.
#[must_use]
pub fn cardkingdom_url(entries: &[&DeckEntry], command_zone: &[(&str, Option<i32>)]) -> String {
    cardkingdom_builder(&deck_cards(entries, command_zone))
}

/// One card's `TCGplayer` link: its product page when the card's `TCGplayer` id is known, else a mass entry of one copy by name.
#[must_use]
pub fn tcgplayer_card_url(name: &str, tcgplayer_id: Option<i32>) -> String {
    match tcgplayer_id {
        Some(id) => format!("https://www.tcgplayer.com/product/{id}"),
        None => tcgplayer_mass(&[(1, name, None)]),
    }
}

/// One card's `CardKingdom` builder link, one copy.
#[must_use]
pub fn cardkingdom_card_url(name: &str) -> String {
    cardkingdom_builder(&[(1, name, None)])
}

/// The deck's `TCGplayer` and `CardKingdom` links: the command zone, then the main and side boards in served order. The maybeboard stays out.
#[must_use]
pub fn links(deck: &Deck) -> (String, String) {
    let zone = command_zone(&deck.deck_profile, &deck.command_zone_cards);
    let entries: Vec<&DeckEntry> = deck
        .entries
        .iter()
        .filter(|e| e.deck_card.board.is_active() || e.deck_card.board.is_sideboard())
        .collect();
    (
        tcgplayer_url(&entries, &zone),
        cardkingdom_url(&entries, &zone),
    )
}

/// The total of a deck page group's `rows` in `currency`, each card's price times its copies;
/// `None` when no card has a price in it.
#[must_use]
pub fn group_price(rows: &[RowInfo], currency: PriceCurrency) -> Option<String> {
    let mut priced = false;
    let mut sum = 0.0_f64;
    for row in rows {
        if let Some(each) = deck_metrics::card_price(&row.card.scryfall_data, currency) {
            priced = true;
            // A row holds at least one copy (`Quantity` is positive; a zone or token row is 1).
            sum += each * f64::from(row.qty.max(1));
        }
    }
    priced.then(|| currency.format_amount(sum))
}

/// The cards a deck page's group sends to a store, each with its copies and its printing's
/// `TCGplayer` id: every card but a token.
#[must_use]
pub fn group_cards(rows: &[RowInfo]) -> Vec<StoreCard<'_>> {
    rows.iter()
        .filter(|row| row.kind != RowKind::Token)
        .map(|row| {
            let data = &row.card.scryfall_data;
            (row.qty, data.name.as_str(), data.tcgplayer_id)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::deck::{Board, Quantity};

    use super::*;
    use crate::test_support::{card, deck, deck_profile};

    #[test]
    fn tcgplayer_gets_each_printing_and_card_kingdom_each_name() {
        let links = cards_links([
            (2, "Sol Ring", Some(1263)),
            (1, "Delver of Secrets // Insectile Aberration", None),
        ]);
        assert_eq!(
            links,
            [
                BuyLink {
                    label: TCGPLAYER,
                    url: "https://www.tcgplayer.com/massentry?productline=Magic&c=2-1263||1+Delver+of+Secrets"
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
        built.entries[0].card.scryfall_data.tcgplayer_id = Some(1263);
        built.entries[1].card.scryfall_data.tcgplayer_id = None;
        built.entries[1].deck_card.quantity = Quantity::new(4).unwrap();
        built.entries[1].deck_card.board = Board::Sideboard;
        built.entries[2].deck_card.board = Board::Maybeboard;
        let mut krenko = card("Krenko, Mob Boss");
        krenko.scryfall_data.tcgplayer_id = Some(9999);
        built.command_zone_cards = vec![krenko];
        let (tcg, ck) = links(&built);
        assert_eq!(
            tcg,
            "https://www.tcgplayer.com/massentry?productline=Magic&c=1-9999||1-1263||4+Lightning+Bolt",
            "each printing by its TCGplayer product, by name where it has none"
        );
        assert_eq!(
            ck,
            "https://www.cardkingdom.com/builder?c=1+Krenko%2C+Mob+Boss%0A1+Sol+Ring%0A4+Lightning+Bolt"
        );
    }

    #[test]
    fn the_deck_and_a_card_offer_both_stores_in_one_order() {
        let built = deck(deck_profile("Krenko"), vec![card("Sol Ring")]);
        let (tcg, ck) = links(&built);
        assert_eq!(deck_links(&built), store_links(tcg, ck));
        assert_eq!(
            card_links("Sol Ring", Some(42)),
            store_links(
                tcgplayer_card_url("Sol Ring", Some(42)),
                cardkingdom_card_url("Sol Ring")
            )
        );
        assert_eq!(
            [CARD_TITLE, DECK_TITLE],
            ["Buy card", "Buy deck"],
            "each dialog names what it buys"
        );
    }

    #[test]
    fn single_card_links_encode_the_front_face_only() {
        assert_eq!(
            tcgplayer_card_url("Fire // Ice", None),
            "https://www.tcgplayer.com/massentry?productline=Magic&c=1+Fire"
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

    #[test]
    fn a_group_sends_its_cards_with_their_copies_and_leaves_tokens_out() {
        use zwipe_core::domain::deck::Board;

        use crate::deck::form::Slot;

        fn row(name: &str, qty: i32, kind: RowKind) -> RowInfo {
            RowInfo {
                card: crate::test_support::card(name),
                qty,
                kind,
            }
        }
        let rows = [
            row("Sol Ring", 1, RowKind::Main { mvp: false }),
            row("Forest", 12, RowKind::Board(Board::Deck)),
            row("Krenko, Mob Boss", 1, RowKind::Zone(Slot::Commander)),
            row("Goblin", 1, RowKind::Token),
        ];
        assert_eq!(
            group_cards(&rows),
            [
                (1, "Sol Ring", Some(91237)),
                (12, "Forest", Some(91237)),
                (1, "Krenko, Mob Boss", Some(91237))
            ],
            "each with its printing's TCGplayer product"
        );
        assert!(
            group_cards(&[row("Goblin", 1, RowKind::Token)]).is_empty(),
            "a group of tokens has nothing to buy"
        );
    }

    #[test]
    fn a_group_s_price_counts_every_copy_and_none_without_prices() {
        let row = |qty: i32| RowInfo {
            card: crate::test_support::card("Forest"),
            qty,
            kind: RowKind::Main { mvp: false },
        };
        assert_eq!(
            group_price(&[row(1), row(12)], PriceCurrency::Usd).as_deref(),
            Some("$1.04"),
            "13 copies at $0.08"
        );
        let mut unpriced = row(1);
        unpriced.card.scryfall_data.prices.usd = None;
        assert_eq!(group_price(&[unpriced], PriceCurrency::Usd), None);
        assert_eq!(group_price(&[], PriceCurrency::Usd), None);
    }
}
