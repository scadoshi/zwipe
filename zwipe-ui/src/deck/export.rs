//! The decklist the Export screen copies, built as the app builds it so it round-trips through Import.
//!
//! Each line is `{qty} {name} ({SET}) {collector number}`, the form Moxfield
//! and MTGA write, so an import lands the exact printing.

use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::deck::{Board, Deck, DeckEntry};
use zwipe_core::http::contracts::deck::HttpSharedDeck;

/// Which boards the export includes. At least one is always on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boards {
    /// The main deck.
    pub deck: bool,
    /// The maybeboard.
    pub maybe: bool,
    /// The sideboard.
    pub side: bool,
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
    pub fn toggle_deck(&mut self) {
        if self.deck && !self.maybe && !self.side {
            return;
        }
        self.deck = !self.deck;
    }

    /// Turns the maybeboard on or off; turning off the last one on turns the deck back on.
    pub fn toggle_maybe(&mut self) {
        self.maybe = !self.maybe;
        self.keep_one();
    }

    /// Turns the sideboard on or off; turning off the last one on turns the deck back on.
    pub fn toggle_side(&mut self) {
        self.side = !self.side;
        self.keep_one();
    }

    fn keep_one(&mut self) {
        if !self.deck && !self.maybe && !self.side {
            self.deck = true;
        }
    }
}

/// `{qty} {name} ({SET}) {number}` for one printing.
fn line(quantity: i32, printing: &ScryfallData) -> String {
    let set = printing.set.to_uppercase();
    format!(
        "{quantity} {} ({set}) {}",
        printing.name, printing.collector_number
    )
}

/// One line for each entry on `board`, in served order.
fn board_lines(entries: &[DeckEntry], board: Board) -> Vec<String> {
    entries
        .iter()
        .filter(|e| e.deck_card.board == board)
        .map(|e| line(*e.deck_card.quantity, &e.card.scryfall_data))
        .collect()
}

/// The decklist: the command zone, then each included board that has cards, joined by newlines.
#[must_use]
pub fn text(deck: &Deck, boards: Boards) -> String {
    let profile = &deck.deck_profile;
    // A command-zone card the deck carries writes its printing; one it
    // doesn't writes the profile's name alone.
    let slot = |id: Option<Uuid>, name: Option<&str>| -> Option<String> {
        let printing = id.and_then(|id| {
            deck.command_zone_cards
                .iter()
                .find(|card| card.scryfall_data.id == id)
        });
        match (printing, name) {
            (Some(card), _) => Some(line(1, &card.scryfall_data)),
            (None, Some(name)) => Some(format!("1 {name}")),
            (None, None) => None,
        }
    };
    let command_zone: Vec<String> = [
        slot(profile.commander_id, profile.commander_name.as_deref()),
        slot(
            profile.partner_commander_id,
            profile.partner_commander_name.as_deref(),
        ),
        slot(profile.background_id, profile.background_name.as_deref()),
        slot(
            profile.signature_spell_id,
            profile.signature_spell_name.as_deref(),
        ),
    ]
    .into_iter()
    .flatten()
    .collect();
    decklist(command_zone, &deck.entries, boards)
}

/// A shared deck's decklist, in the same form as [`text`].
#[must_use]
pub fn shared_text(deck: &HttpSharedDeck, boards: Boards) -> String {
    let command_zone: Vec<String> = [
        &deck.commander,
        &deck.partner_commander,
        &deck.background,
        &deck.signature_spell,
    ]
    .into_iter()
    .flatten()
    .map(|card| line(1, &card.scryfall_data))
    .collect();
    decklist(command_zone, &deck.entries, boards)
}

/// The command zone's lines under their heading, then each included board that has cards.
fn decklist(command_zone: Vec<String>, entries: &[DeckEntry], boards: Boards) -> String {
    let mut lines: Vec<String> = Vec::new();
    if !command_zone.is_empty() {
        lines.push("// Commander".to_owned());
        lines.extend(command_zone);
        lines.push(String::new());
    }
    if boards.deck {
        let cards = board_lines(entries, Board::Deck);
        if !cards.is_empty() {
            lines.push("// Deck".to_owned());
            lines.extend(cards);
        }
    }
    for (on, board, heading) in [
        (boards.maybe, Board::Maybeboard, "// Maybeboard"),
        (boards.side, Board::Sideboard, "// Sideboard"),
    ] {
        let cards = board_lines(entries, board);
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
    use zwipe_core::domain::deck::requests::import_deck_cards::{ImportDeckCards, NamedPrinting};
    use zwipe_core::domain::deck::{ImportMode, Quantity};

    use super::*;

    #[test]
    fn a_shared_deck_writes_its_commander_then_its_deck_as_the_owner_s_does() {
        let commander = crate::test_support::card("Krenko, Mob Boss");
        let held = crate::test_support::deck(
            crate::test_support::deck_profile("Goblins"),
            vec![crate::test_support::card("Goblin Guide")],
        );
        let shared: HttpSharedDeck = serde_json::from_value(serde_json::json!({
            "name": "Goblins",
            "format": "commander",
            "commander": commander,
            "partner_commander": null,
            "background": null,
            "signature_spell": null,
            "entries": held.entries,
        }))
        .unwrap();
        let list = shared_text(&shared, Boards::default());
        let lines: Vec<&str> = list.lines().collect();
        assert_eq!(lines.first().copied(), Some("// Commander"), "{list}");
        assert!(
            lines
                .get(1)
                .is_some_and(|l| l.starts_with("1 Krenko, Mob Boss (")),
            "{list}"
        );
        assert!(list.contains("// Deck\n1 Goblin Guide ("), "{list}");
    }
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
            "// Commander\n1 Krenko, Mob Boss\n\n// Deck\n2 Sol Ring (M15) 77"
        );
        let all = Boards {
            deck: true,
            maybe: true,
            side: true,
        };
        assert_eq!(
            text(&built, all),
            "// Commander\n1 Krenko, Mob Boss\n\n// Deck\n2 Sol Ring (M15) 77\n\n// Maybeboard\n1 Opt (M15) 77\n\n// Sideboard\n1 Duress (M15) 77"
        );
    }

    #[test]
    fn a_carried_command_zone_card_writes_its_printing() {
        let commander = card("Krenko, Mob Boss");
        let mut profile = deck_profile("Krenko");
        profile.commander_id = Some(commander.scryfall_data.id);
        profile.commander_name = Some("Krenko, Mob Boss".to_owned());
        let mut built = deck(profile, Vec::new());
        built.command_zone_cards = vec![commander];
        assert_eq!(
            text(&built, Boards::default()),
            "// Commander\n1 Krenko, Mob Boss (M15) 77\n"
        );
    }

    #[test]
    fn the_export_imports_back_as_the_same_printings() {
        let mut delver = card("Delver of Secrets");
        delver.scryfall_data.set = "isd".to_owned();
        delver.scryfall_data.collector_number = "51a".to_owned();
        let mut keeper = card("Bloodline Keeper // Lord of Lineage");
        keeper.scryfall_data.set = "isd".to_owned();
        keeper.scryfall_data.collector_number = "90".to_owned();
        let built = deck(deck_profile("Vampires"), vec![delver, keeper]);
        let parsed = ImportDeckCards::parse(
            Uuid::nil(),
            Uuid::nil(),
            &text(&built, Boards::default()),
            /* email_verified */ true,
            None,
            ImportMode::Replace,
        )
        .unwrap();
        let lines: Vec<(&str, Option<NamedPrinting>)> = parsed
            .lines
            .iter()
            .map(|l| (l.card_name.as_str(), l.printing.clone()))
            .collect();
        let printing = |number: &str| {
            Some(NamedPrinting {
                set_code: "isd".to_owned(),
                collector_number: Some(number.to_owned()),
            })
        };
        assert_eq!(
            lines,
            vec![
                ("Delver of Secrets", printing("51a")),
                ("Bloodline Keeper // Lord of Lineage", printing("90")),
            ]
        );
    }

    #[test]
    fn empty_boards_and_a_missing_command_zone_leave_no_headings() {
        let built = deck(deck_profile("Empty"), Vec::new());
        assert_eq!(text(&built, Boards::default()), "");
    }
}
