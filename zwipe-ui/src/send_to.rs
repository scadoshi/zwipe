//! Send to: a card added to another of the user's decks, on the board the user picks, from
//! wherever the card shows. The sheet leads with the boards as chips and lists the decks under
//! them; a tap on a deck sends the card there.

use uuid::Uuid;
use zwipe_core::domain::deck::{Board, DeckEntry, DeckProfile};

/// The sheet's title, and the label of the button that opens it.
pub const TITLE: &str = "Send to";

/// The search field's placeholder.
pub const SEARCH_PLACEHOLDER: &str = "Search decks";

/// The list's line while the decks load.
pub const LOADING: &str = "Loading decks";

/// The list's line when the user has no other deck.
pub const NO_DECKS: &str = "No other decks yet";

/// The list's line when the search matches no deck.
pub const NO_MATCH: &str = "No deck matches";

/// The boards, in the order their chips run.
pub const BOARDS: [Board; 3] = [Board::Deck, Board::Maybeboard, Board::Sideboard];

/// A board's chip.
#[must_use]
pub const fn chip_label(board: Board) -> &'static str {
    match board {
        Board::Deck => "Main",
        Board::Maybeboard => "Maybe",
        Board::Sideboard => "Side",
    }
}

/// A board as the toast names it, the way the card's board moves do.
#[must_use]
pub const fn board_name(board: Board) -> &'static str {
    match board {
        Board::Deck => "mainboard",
        Board::Maybeboard => "maybeboard",
        Board::Sideboard => "sideboard",
    }
}

/// The toast once the card is in `deck` on `board`.
#[must_use]
pub fn sent(deck: &str, board: Board) -> String {
    format!("Sent to {deck}, {}", board_name(board))
}

/// The toast when `deck` already holds the card, naming the deck it went to rather than the
/// one the user is on, and the board that holds it when that is known. A deck holds one copy
/// of a card across its boards.
#[must_use]
pub fn already_in(deck: &str, board: Option<Board>) -> String {
    match board {
        Some(board) => format!("Already in {deck}'s {}", board_name(board)),
        None => format!("Already in {deck}"),
    }
}

/// The board of `entries` that holds the card with `oracle_id`, if any.
#[must_use]
pub fn holding_board(entries: &[DeckEntry], oracle_id: Uuid) -> Option<Board> {
    entries
        .iter()
        .find(|e| e.deck_card.oracle_id == oracle_id)
        .map(|e| e.deck_card.board)
}

/// A deck the card can go to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The deck.
    pub id: Uuid,
    /// Its name.
    pub name: String,
}

impl From<&DeckProfile> for Target {
    fn from(deck: &DeckProfile) -> Self {
        Self {
            id: deck.id,
            name: deck.name.to_string(),
        }
    }
}

/// The decks to list: every deck but `current`, the one the user is on, if any, whose name
/// holds `query`, ignoring case and the space around it, in the order given.
#[must_use]
pub fn targets(decks: &[Target], current: Option<Uuid>, query: &str) -> Vec<Target> {
    let wanted = query.trim().to_lowercase();
    decks
        .iter()
        .filter(|deck| Some(deck.id) != current)
        .filter(|deck| wanted.is_empty() || deck.name.to_lowercase().contains(&wanted))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck(n: u128, name: &str) -> Target {
        Target {
            id: Uuid::from_u128(n),
            name: name.to_owned(),
        }
    }

    fn names(decks: &[Target]) -> Vec<&str> {
        decks.iter().map(|d| d.name.as_str()).collect()
    }

    #[test]
    fn the_deck_the_user_is_on_is_left_out() {
        let decks = [deck(1, "Krenko"), deck(2, "Yuriko"), deck(3, "Atraxa")];
        assert_eq!(
            names(&targets(&decks, Some(Uuid::from_u128(2)), "")),
            ["Krenko", "Atraxa"]
        );
        assert_eq!(
            names(&targets(&decks, None, "")),
            ["Krenko", "Yuriko", "Atraxa"]
        );
    }

    #[test]
    fn the_search_matches_any_part_of_a_name_in_any_case() {
        let decks = [deck(1, "Krenko Goblins"), deck(2, "Yuriko Ninjas")];
        assert_eq!(names(&targets(&decks, None, "  gob ")), ["Krenko Goblins"]);
        assert_eq!(targets(&decks, None, "elves"), Vec::<Target>::new());
    }

    #[test]
    fn the_chips_run_main_maybe_side() {
        assert_eq!(BOARDS.map(chip_label), ["Main", "Maybe", "Side"]);
    }

    #[test]
    fn a_card_the_deck_already_holds_names_that_deck_and_its_board() {
        assert_eq!(
            already_in("Krenko", Some(Board::Maybeboard)),
            "Already in Krenko's maybeboard"
        );
        assert_eq!(
            already_in("Krenko", Some(Board::Deck)),
            "Already in Krenko's mainboard"
        );
        assert_eq!(
            already_in("Krenko", Some(Board::Sideboard)),
            "Already in Krenko's sideboard"
        );
        assert_eq!(
            already_in("Surfer 399 Isshin", None),
            "Already in Surfer 399 Isshin",
            "the board unknown"
        );
    }

    #[test]
    fn the_board_holding_a_card_is_found_by_its_oracle_id() {
        let held = crate::test_support::deck(
            crate::test_support::deck_profile("Krenko"),
            vec![
                crate::test_support::card("Island"),
                crate::test_support::card("Sol Ring"),
            ],
        );
        let mut entries = held.entries;
        let oracle = entries.get(1).map(|e| e.deck_card.oracle_id).unwrap();
        if let Some(sol_ring) = entries.get_mut(1) {
            sol_ring.deck_card.board = Board::Maybeboard;
        }
        assert_eq!(holding_board(&entries, oracle), Some(Board::Maybeboard));
        assert_eq!(
            holding_board(&entries, Uuid::nil()),
            None,
            "not in the deck"
        );
    }

    #[test]
    fn the_toast_names_the_deck_and_the_board() {
        assert_eq!(
            sent("Krenko", Board::Maybeboard),
            "Sent to Krenko, maybeboard"
        );
        assert_eq!(sent("Krenko", Board::Deck), "Sent to Krenko, mainboard");
        assert_eq!(
            sent("Krenko", Board::Sideboard),
            "Sent to Krenko, sideboard"
        );
    }
}
