//! Generators shared by the property tests.

use proptest::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{Board, DeckCard, DeckEntry, Quantity};

use crate::test_support::card;

/// A card with a short name, a land or not.
pub(super) fn any_card() -> impl Strategy<Value = Card> {
    ("[abc]{1,4}", any::<bool>()).prop_map(|(name, land)| {
        let mut card = card(&name);
        card.scryfall_data.type_line = Some(
            if land {
                "Basic Land — Forest"
            } else {
                "Creature — Elf"
            }
            .to_owned(),
        );
        card
    })
}

/// Any board.
pub(super) fn board() -> impl Strategy<Value = Board> {
    prop::sample::select(vec![Board::Deck, Board::Maybeboard, Board::Sideboard])
}

/// A copy count, mostly small and sometimes at the top of the range.
pub(super) fn copies() -> impl Strategy<Value = i32> {
    prop_oneof![1..=10i32, (Quantity::MAX - 4)..=Quantity::MAX]
}

/// A deck card: any card on any board holding `copies()`.
pub(super) fn entry() -> impl Strategy<Value = DeckEntry> {
    (any_card(), board(), copies()).prop_map(|(card, board, copies)| DeckEntry {
        deck_card: DeckCard {
            deck_id: uuid::Uuid::nil(),
            scryfall_data_id: card.scryfall_data.id,
            oracle_id: card.scryfall_data.oracle_id.unwrap(),
            quantity: Quantity::new(copies).unwrap(),
            board,
            mvp_at: None,
            flagged: false,
        },
        card,
    })
}
