//! Whether a deck card may take one more copy: a card holds at most [`Quantity::MAX`], and a
//! deck's total, every copy on every board plus each filled command zone slot, stays within the
//! account's card limit. A card or deck already past either keeps its count and can only go down.

use zwipe_core::domain::deck::{
    DeckEntry, MAX_CARDS_PER_DECK, Quantity, UNVERIFIED_MAX_CARDS_PER_DECK,
};

/// The card limit for an account, the lower one while its email isn't verified.
#[must_use]
pub const fn card_limit(email_verified: bool) -> i64 {
    if email_verified {
        MAX_CARDS_PER_DECK
    } else {
        UNVERIFIED_MAX_CARDS_PER_DECK
    }
}

/// A deck's total toward its limit: every copy in `entries`, on every board, plus
/// `zone_cards`, its filled command zone slots.
#[must_use]
pub fn deck_total(entries: &[DeckEntry], zone_cards: usize) -> i64 {
    let copies: i64 = entries
        .iter()
        .map(|entry| i64::from(*entry.deck_card.quantity))
        .sum();
    copies.saturating_add(i64::try_from(zone_cards).unwrap_or(i64::MAX))
}

/// Whether a card holding `quantity` copies may take one more in a deck at `total` of `limit`.
#[must_use]
pub fn can_add_copy(quantity: i32, total: i64, limit: i64) -> bool {
    quantity < Quantity::MAX && total < limit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    #[test]
    fn a_card_takes_copies_up_to_99() {
        assert!(can_add_copy(98, 10, 100));
        assert!(!can_add_copy(99, 10, 100));
        assert!(!can_add_copy(120, 10, 100), "a card past 99 only goes down");
    }

    #[test]
    fn a_deck_takes_copies_up_to_its_limit() {
        assert!(can_add_copy(1, 99, 100));
        assert!(!can_add_copy(1, 100, 100));
        assert!(
            !can_add_copy(1, 130, 100),
            "a deck past its limit only goes down"
        );
    }

    #[test]
    fn an_unverified_account_has_the_lower_limit() {
        assert_eq!(card_limit(true), 500);
        assert_eq!(card_limit(false), 100);
    }

    #[test]
    fn every_board_and_each_command_zone_card_count_toward_the_total() {
        let mut deck = test_support::deck(
            test_support::deck_profile("Copies"),
            vec![test_support::card("Elf"), test_support::card("Bolt")],
        );
        if let Some(bolt) = deck.entries.get_mut(1) {
            bolt.deck_card.quantity = Quantity::new(4).unwrap();
            bolt.deck_card.board = zwipe_core::domain::deck::Board::Sideboard;
        }
        assert_eq!(deck_total(&deck.entries, 0), 5);
        assert_eq!(deck_total(&deck.entries, 2), 7);
    }
}
