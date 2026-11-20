//! Send to: a card added to another of the user's decks, on the board the user picks, from
//! wherever the card shows. The sheet leads with the boards as chips and lists the decks under
//! them; a tap on a deck sends the card there.

use uuid::Uuid;
use zwipe_core::domain::deck::{Board, DeckCard, DeckProfile, Quantity};

use crate::stores::undo::UndoAction;

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
/// one the user is on. A deck holds one copy of a card across its boards, and the add's answer
/// doesn't say which, so the toast shows as the answer lands without naming one.
#[must_use]
pub fn already_in(deck: &str) -> String {
    format!("Already in {deck}")
}

/// The copies a send adds.
pub const COPIES: i32 = 1;

/// The undo entry for sending `card_name` (printing `card_id`) to `target`'s `board` from the
/// deck `from`, the deck whose history it goes on: `None` when the send came from outside a deck.
#[must_use]
pub fn undo_entry(
    from: Option<Uuid>,
    target: &Target,
    board: Board,
    card_id: Uuid,
    card_name: &str,
) -> Option<(Uuid, UndoAction)> {
    from.map(|from| {
        (
            from,
            UndoAction::SentTo {
                card_id,
                card_name: card_name.to_owned(),
                deck_id: target.id,
                deck_name: target.name.clone(),
                board,
                copies: COPIES,
            },
        )
    })
}

/// The toast when undoing a send finds the card moved boards or gone from `deck`, the deck it
/// went to, and leaves it.
#[must_use]
pub fn left_alone(deck: &str) -> String {
    let deck = deck.trim();
    if deck.is_empty() {
        "Already changed in that deck, left as is".to_owned()
    } else {
        format!("Already changed in {deck}, left as is")
    }
}

/// What undoing a send does to the card's row in the deck it went to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendUndo {
    /// The row keeps more than the send added: it comes down to this many copies.
    Lower(Quantity),
    /// The row holds no more than the send added: it goes.
    Remove,
}

/// How undo takes back the `copies` a send put on `board`, given the card's row there now:
/// only those copies, and only while the row is still on that board. `None` when the row left
/// or moved boards, which undo leaves alone.
#[must_use]
pub fn send_undo(row: Option<&DeckCard>, board: Board, copies: i32) -> Option<SendUndo> {
    let row = row.filter(|row| row.board == board)?;
    Some(
        Quantity::new(row.quantity.saturating_sub(copies))
            .map_or(SendUndo::Remove, SendUndo::Lower),
    )
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

/// The deck Enter in the search sends to for the text `typed`: the first of [`targets`] for that
/// text while it has any, so Enter that lands before the list redraws still acts on it.
#[must_use]
pub fn target_on_enter(decks: &[Target], current: Option<Uuid>, typed: &str) -> Option<Target> {
    if typed.trim().is_empty() {
        None
    } else {
        targets(decks, current, typed).into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(board: Board, copies: i32) -> DeckCard {
        DeckCard {
            deck_id: Uuid::from_u128(2),
            scryfall_data_id: Uuid::from_u128(9),
            oracle_id: Uuid::from_u128(10),
            quantity: Quantity::new(copies).unwrap(),
            board,
            mvp_at: None,
            flagged: false,
        }
    }

    #[test]
    fn undo_takes_back_only_the_copy_it_sent() {
        let raised = row(Board::Maybeboard, 3);
        assert_eq!(
            send_undo(Some(&raised), Board::Maybeboard, 1),
            Some(SendUndo::Lower(Quantity::new(2).unwrap())),
            "raised to 3 since, it ends at 2"
        );
        assert_eq!(
            send_undo(Some(&row(Board::Maybeboard, 1)), Board::Maybeboard, 1),
            Some(SendUndo::Remove),
            "the copy it sent is all there is, so the row goes"
        );
    }

    #[test]
    fn a_card_left_alone_says_which_deck_it_changed_in() {
        assert_eq!(
            left_alone("Krenko"),
            "Already changed in Krenko, left as is"
        );
        assert_eq!(left_alone("  "), "Already changed in that deck, left as is");
    }

    #[test]
    fn undo_leaves_a_card_that_moved_boards_or_left() {
        assert_eq!(
            send_undo(Some(&row(Board::Deck, 3)), Board::Maybeboard, 1),
            None
        );
        assert_eq!(send_undo(None, Board::Maybeboard, 1), None);
    }

    #[test]
    fn a_send_from_a_deck_goes_on_that_deck_s_undo() {
        let (from, card) = (Uuid::from_u128(1), Uuid::from_u128(9));
        let target = deck(2, "Krenko");
        assert_eq!(
            undo_entry(Some(from), &target, Board::Maybeboard, card, "Sol Ring"),
            Some((
                from,
                UndoAction::SentTo {
                    card_id: card,
                    card_name: "Sol Ring".to_owned(),
                    deck_id: target.id,
                    deck_name: "Krenko".to_owned(),
                    board: Board::Maybeboard,
                    copies: 1,
                }
            ))
        );
    }

    #[test]
    fn a_send_from_outside_a_deck_records_no_undo() {
        assert_eq!(
            undo_entry(
                None,
                &deck(2, "Krenko"),
                Board::Maybeboard,
                Uuid::from_u128(9),
                "Sol Ring"
            ),
            None
        );
    }

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
    fn a_card_the_deck_already_holds_names_that_deck() {
        assert_eq!(already_in("Krenko"), "Already in Krenko");
        assert_eq!(
            already_in("Surfer 399 Isshin"),
            "Already in Surfer 399 Isshin"
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

    /// Enter reads the deck search as typed, so it sends to that text's first deck, never the
    /// first deck drawn for the text before it.
    #[test]
    fn enter_sends_to_the_first_deck_for_the_text_as_typed() {
        let decks = vec![deck(1, "Atraxa"), deck(2, "Krenko"), deck(3, "Kraum")];
        assert_eq!(
            targets(&decks, None, "kr").first(),
            Some(&decks[1]),
            "the stale top match"
        );
        assert_eq!(
            target_on_enter(&decks, None, "krau"),
            Some(decks[2].clone())
        );
        assert_eq!(
            target_on_enter(&decks, Some(decks[2].id), "krau"),
            None,
            "never the deck it's on"
        );
        assert_eq!(
            target_on_enter(&decks, None, " "),
            None,
            "a blank search has no target"
        );
    }
}
