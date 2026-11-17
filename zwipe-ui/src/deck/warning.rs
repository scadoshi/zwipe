//! Which one-tap fix a deck warning offers, sorted out the way the app's `deck_warnings.rs` does it.

use std::collections::HashSet;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::card_price;
use zwipe_core::domain::deck::{DeckEntry, DeckWarning, WarningAction};

/// A warning's fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fix {
    /// Too few cards: "Add cards" or "Import".
    BelowMinimum,
    /// Sets the card's quantity to this many: "Fix to {n}".
    Quantity {
        /// The card's printing ID.
        card: Uuid,
        /// The quantity to set.
        quantity: i32,
    },
    /// Clears the commander: "Clear".
    ClearCommander,
    /// Takes the card out of the deck: "Remove".
    Remove {
        /// The card's printing ID.
        card: Uuid,
    },
}

/// The fix `warning` offers, or `None` for a deck-level warning with nothing to tap.
///
/// The below-minimum warning is matched on its text, as in the app, so the wire format stays as it is.
#[must_use]
pub fn fix(warning: &DeckWarning) -> Option<Fix> {
    let Some(card) = warning.scryfall_data_id() else {
        return warning
            .contains("requires at least")
            .then_some(Fix::BelowMinimum);
    };
    Some(match warning.action() {
        Some(WarningAction::FixQuantity(quantity)) => Fix::Quantity {
            card,
            quantity: *quantity,
        },
        Some(WarningAction::ClearCommander) => Fix::ClearCommander,
        Some(WarningAction::Remove) | None => Fix::Remove { card },
    })
}

/// The labels of a fix's buttons, in order.
#[must_use]
pub fn labels(fix: Fix) -> Vec<String> {
    match fix {
        Fix::BelowMinimum => vec!["Add cards".to_owned(), "Import".to_owned()],
        Fix::Quantity { quantity, .. } => vec![format!("Fix to {quantity}")],
        Fix::ClearCommander => vec!["Clear".to_owned()],
        Fix::Remove { .. } => vec!["Remove".to_owned()],
    }
}

/// The Unpriced warning's button while its cards are hidden.
pub const SHOW_UNPRICED: &str = "Show";

/// The Unpriced warning's button while its cards show.
pub const HIDE_UNPRICED: &str = "Hide";

/// The deck's cards with no price in `currency`, one per printing in deck order: the mainboard's,
/// then each command-zone card not also on the mainboard. These are the copies the Budget's total
/// and average leave out, as [`DeckMetrics`](zwipe_core::domain::deck::DeckMetrics) counts them.
#[must_use]
pub fn unpriced(
    entries: &[DeckEntry],
    command_zone: &[Card],
    currency: PriceCurrency,
) -> Vec<Card> {
    let main: Vec<&Card> = entries
        .iter()
        .filter(|e| e.deck_card.board.is_active())
        .map(|e| &e.card)
        .collect();
    let zone = command_zone.iter().filter(|c| {
        !main
            .iter()
            .any(|m| m.scryfall_data.id == c.scryfall_data.id)
    });
    let mut seen = HashSet::new();
    main.iter()
        .copied()
        .chain(zone)
        .filter(|c| card_price(&c.scryfall_data, currency).is_none())
        .filter(|c| seen.insert(c.scryfall_data.id))
        .cloned()
        .collect()
}

/// The Unpriced warning's line for `count` cards with no price in `currency`.
#[must_use]
pub fn unpriced_line(count: usize, currency: PriceCurrency) -> String {
    let label = currency.label();
    if count == 1 {
        format!("1 card has no {label} price")
    } else {
        format!("{count} cards have no {label} price")
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::deck::{Board, DeckMetrics};

    use super::*;
    use crate::test_support;

    #[test]
    fn each_warning_gets_the_apps_fix() {
        let card = Uuid::new_v4();
        let short = DeckWarning::new("deck has 60 cards, commander requires at least 100");
        assert_eq!(fix(&short), Some(Fix::BelowMinimum));
        assert_eq!(labels(Fix::BelowMinimum), ["Add cards", "Import"]);
        assert_eq!(
            fix(&DeckWarning::new(
                "deck has 40 lands, below the land target of 41"
            )),
            None
        );
        let copies = DeckWarning::with_action(
            "Sol Ring has 2 copies, Commander allows 1",
            card,
            WarningAction::FixQuantity(1),
        );
        assert_eq!(fix(&copies), Some(Fix::Quantity { card, quantity: 1 }));
        assert_eq!(labels(Fix::Quantity { card, quantity: 1 }), ["Fix to 1"]);
        let commander = DeckWarning::with_action(
            "Opt is not a valid commander for commander",
            card,
            WarningAction::ClearCommander,
        );
        assert_eq!(fix(&commander), Some(Fix::ClearCommander));
        let banned = DeckWarning::with_card("Mana Crypt is banned in commander", card);
        assert_eq!(fix(&banned), Some(Fix::Remove { card }));
    }

    /// `name` with `usd` as its only price.
    fn priced(name: &str, usd: Option<&str>) -> Card {
        let mut card = test_support::card(name);
        let prices = &mut card.scryfall_data.prices;
        prices.usd = usd.map(str::to_owned);
        prices.eur = None;
        prices.tix = None;
        card
    }

    #[test]
    fn the_unpriced_are_the_cards_the_budget_leaves_out() {
        let sol_ring = priced("Sol Ring", Some("1.50"));
        let mystery = priced("Mystery Card", None);
        let sideboard = priced("Sideboard Oddity", None);
        let commander = priced("Unpriced Commander", None);
        let mut deck = test_support::deck(
            test_support::deck_profile("Deck"),
            vec![sol_ring, mystery.clone(), sideboard],
        );
        if let Some(side) = deck.entries.get_mut(2) {
            side.deck_card.board = Board::Sideboard;
        }
        // The command zone's copy of a card already on the mainboard counts once.
        let zone = vec![commander, mystery];
        let names = |cards: Vec<Card>| -> Vec<String> {
            cards.into_iter().map(|c| c.scryfall_data.name).collect()
        };
        assert_eq!(
            names(unpriced(&deck.entries, &zone, PriceCurrency::Usd)),
            ["Mystery Card", "Unpriced Commander"]
        );
        let metrics = DeckMetrics::from_entries_and_command_zone(&deck.entries, &zone);
        assert_eq!(
            unpriced(&deck.entries, &zone, PriceCurrency::Usd).len(),
            metrics.unpriced(PriceCurrency::Usd),
            "one copy each, so the list is the Budget's count"
        );
        assert_eq!(
            names(unpriced(&deck.entries, &zone, PriceCurrency::Eur)),
            ["Sol Ring", "Mystery Card", "Unpriced Commander"],
            "the shown currency decides"
        );
    }

    #[test]
    fn the_unpriced_line_counts_cards_in_the_shown_currency() {
        assert_eq!(
            unpriced_line(1, PriceCurrency::Usd),
            "1 card has no USD price"
        );
        assert_eq!(
            unpriced_line(3, PriceCurrency::Eur),
            "3 cards have no EUR price"
        );
    }
}
