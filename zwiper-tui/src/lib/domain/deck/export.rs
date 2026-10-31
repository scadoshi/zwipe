//! The decklist the Export screen copies, built as the app builds it so it round-trips through Import.

use zwipe_core::domain::deck::{Board, Deck};

/// Which boards the export includes. At least one is always on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Boards {
    pub(crate) deck: bool,
    pub(crate) maybe: bool,
    pub(crate) side: bool,
}

impl Default for Boards {
    /// Only the deck, as the screen opens.
    fn default() -> Self {
        Self {
            deck: true,
            maybe: false,
            side: false,
        }
    }
}

impl Boards {
    /// Turns the deck board on or off; turning it off does nothing while it is the only one on.
    pub(crate) fn toggle_deck(&mut self) {
        if self.deck && !self.maybe && !self.side {
            return;
        }
        self.deck = !self.deck;
    }

    /// Turns the maybeboard on or off; turning off the last one on turns the deck back on.
    pub(crate) fn toggle_maybe(&mut self) {
        self.maybe = !self.maybe;
        self.keep_one();
    }

    /// Turns the sideboard on or off; turning off the last one on turns the deck back on.
    pub(crate) fn toggle_side(&mut self) {
        self.side = !self.side;
        self.keep_one();
    }

    fn keep_one(&mut self) {
        if !self.deck && !self.maybe && !self.side {
            self.deck = true;
        }
    }
}

/// `{qty} {name}` for each entry on `board`, in served order.
fn board_lines(deck: &Deck, board: Board) -> Vec<String> {
    deck.entries
        .iter()
        .filter(|e| e.deck_card.board == board)
        .map(|e| format!("{} {}", *e.deck_card.quantity, e.card.scryfall_data.name))
        .collect()
}

/// The decklist: the command zone, then each included board that has cards, joined by newlines.
pub(crate) fn text(deck: &Deck, boards: Boards) -> String {
    let profile = &deck.deck_profile;
    let mut lines: Vec<String> = Vec::new();
    let command_zone: Vec<&str> = [
        profile.commander_name.as_deref(),
        profile.partner_commander_name.as_deref(),
        profile.background_name.as_deref(),
        profile.signature_spell_name.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !command_zone.is_empty() {
        lines.push("// Commander".to_owned());
        lines.extend(command_zone.iter().map(|name| format!("1 {name}")));
        lines.push(String::new());
    }
    if boards.deck {
        let cards = board_lines(deck, Board::Deck);
        if !cards.is_empty() {
            lines.push("// Deck".to_owned());
            lines.extend(cards);
        }
    }
    for (on, board, heading) in [
        (boards.maybe, Board::Maybeboard, "// Maybeboard"),
        (boards.side, Board::Sideboard, "// Sideboard"),
    ] {
        let cards = board_lines(deck, board);
        if on && !cards.is_empty() {
            lines.push(String::new());
            lines.push(heading.to_owned());
            lines.extend(cards);
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::deck::Quantity;

    use super::*;
    use crate::test_support::{card, deck, deck_profile};

    #[test]
    fn at_least_one_board_stays_on() {
        let mut boards = Boards::default();
        boards.toggle_deck();
        assert!(boards.deck, "the only board on stays on");
        boards.toggle_maybe();
        boards.toggle_deck();
        assert_eq!((boards.deck, boards.maybe), (false, true));
        boards.toggle_maybe();
        assert_eq!(
            (boards.deck, boards.maybe),
            (true, false),
            "the deck comes back"
        );
        boards.toggle_side();
        boards.toggle_side();
        assert_eq!(boards, Boards::default());
    }

    #[test]
    fn the_text_lists_the_command_zone_then_each_board() {
        let mut profile = deck_profile("Krenko");
        profile.commander_name = Some("Krenko, Mob Boss".to_owned());
        let mut built = deck(profile, vec![card("Sol Ring"), card("Opt"), card("Duress")]);
        built.entries[0].deck_card.quantity = Quantity::new(2).unwrap();
        built.entries[1].deck_card.board = Board::Maybeboard;
        built.entries[2].deck_card.board = Board::Sideboard;
        assert_eq!(
            text(&built, Boards::default()),
            "// Commander\n1 Krenko, Mob Boss\n\n// Deck\n2 Sol Ring"
        );
        let all = Boards {
            deck: true,
            maybe: true,
            side: true,
        };
        assert_eq!(
            text(&built, all),
            "// Commander\n1 Krenko, Mob Boss\n\n// Deck\n2 Sol Ring\n\n// Maybeboard\n1 Opt\n\n// Sideboard\n1 Duress"
        );
    }

    #[test]
    fn empty_boards_and_a_missing_command_zone_leave_no_headings() {
        let built = deck(deck_profile("Empty"), Vec::new());
        assert_eq!(text(&built, Boards::default()), "");
    }
}
