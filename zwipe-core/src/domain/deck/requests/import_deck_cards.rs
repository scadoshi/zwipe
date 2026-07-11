//! Deck card import operation.
//!
//! Parses plain-text decklists (`[qty] [card name]` per line),
//! resolves card names against the database, and bulk-inserts matched cards.
//! Supports `// Sideboard` and `// Maybeboard` section headers.

use crate::domain::deck::{Board, ImportMode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A parsed line from import text.
#[derive(Debug, Clone)]
pub struct ImportLine {
    /// Number of copies.
    pub quantity: i32,
    /// Card name (as entered by the user).
    pub card_name: String,
    /// Which board this card belongs to.
    pub board: Board,
}

/// Validated import request.
#[derive(Debug, Clone)]
pub struct ImportDeckCards {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck to import into.
    pub deck_id: Uuid,
    /// Parsed import lines.
    pub lines: Vec<ImportLine>,
    /// Whether the requesting user's email is verified.
    pub email_verified: bool,
    /// Add on top of each board, or replace each board present in the import.
    pub mode: ImportMode,
}

impl ImportDeckCards {
    /// Parses plain text into an import request.
    ///
    /// Format: `[qty] [card name]` per line.
    /// Quantity defaults to 1 if the first word is not a number.
    /// Empty and whitespace-only lines are skipped.
    /// When `board_override` is `Some`, every imported line is placed on that
    /// board and section headers in the text are ignored.
    /// With [`ImportMode::Replace`], each board present in the import is
    /// replaced (cards on it not in the import are removed).
    pub fn parse(
        user_id: Uuid,
        deck_id: Uuid,
        text: &str,
        email_verified: bool,
        board_override: Option<Board>,
        mode: ImportMode,
    ) -> Self {
        let mut current_board = board_override.unwrap_or(Board::Deck);
        let mut lines = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Detect section headers: "// Sideboard", "// Maybeboard", "//Sideboard", etc.
            // Skip header detection entirely when a board override is active.
            if board_override.is_none() {
                let header = trimmed.strip_prefix("//").map(|s| s.trim().to_lowercase());
                if let Some(ref h) = header {
                    match h.as_str() {
                        "sideboard" => {
                            current_board = Board::Sideboard;
                            continue;
                        }
                        "maybeboard" => {
                            current_board = Board::Maybeboard;
                            continue;
                        }
                        "deck" => {
                            current_board = Board::Deck;
                            continue;
                        }
                        // Skip other comment headers like "// Commander"
                        _ if !h.is_empty() => continue,
                        _ => {}
                    }
                }
            } else if trimmed.starts_with("//") {
                // With override active, still skip comment lines
                continue;
            }

            let (quantity, rest) = match trimmed.split_once(char::is_whitespace) {
                Some((first, rest)) => {
                    // Handle "4" or "4x" quantity prefix
                    let stripped = first.strip_suffix('x').unwrap_or(first);
                    match stripped.parse::<i32>() {
                        Ok(qty) => (qty, rest.trim()),
                        Err(_) => (1, trimmed),
                    }
                }
                None => (1, trimmed),
            };
            let card_name = strip_trailing_metadata(rest);
            if card_name.is_empty() || quantity < 1 {
                continue;
            }
            lines.push(ImportLine {
                quantity,
                card_name,
                board: current_board,
            });
        }
        Self {
            user_id,
            deck_id,
            lines,
            email_verified,
            mode,
        }
    }
}

/// Strips trailing metadata from a card name.
///
/// Handles formats like:
/// - Moxfield: `"Aether Hub"` (no metadata)
/// - Archidekt: `"Aether Hub (drc) 145 [Land]"` → `"Aether Hub"`
/// - Archidekt foil: `"Dr. Madison Li (pip) 531 *F* [Draw]"` → `"Dr. Madison Li"`
fn strip_trailing_metadata(s: &str) -> String {
    let name = if let Some(idx) = s.find('(') {
        &s[..idx]
    } else if let Some(idx) = s.find('[') {
        &s[..idx]
    } else {
        s
    };
    name.trim().to_string()
}

/// The front face of a double-faced card's canonical name (`"A // B"` -> `"A"`),
/// or `None` for a single-faced name. Import indexes a card under both its full
/// name and this front face, so a decklist entry that names only the front
/// still resolves.
pub fn dfc_front_face(name: &str) -> Option<&str> {
    name.split_once(" // ").map(|(front, _)| front.trim())
}

/// The front face of a user-entered card name: everything before the first
/// slash, trimmed. Collapses `"A // B"`, `"A / B"`, and `"A"` to the same key so
/// all three resolve to the double-faced card. Returns the whole trimmed name
/// when there's no slash.
pub fn entry_front_face(name: &str) -> &str {
    match name.split_once('/') {
        Some((front, _)) => front.trim(),
        None => name.trim(),
    }
}

/// A successfully imported card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedCard {
    /// Card name that was resolved.
    pub name: String,
    /// Number of copies imported.
    pub quantity: i32,
}

/// A card name that couldn't be resolved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnresolvedCard {
    /// The card name from the import text.
    pub name: String,
    /// Why it couldn't be resolved.
    pub reason: String,
}

/// Import operation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDeckCardsResult {
    /// Cards that were successfully imported.
    pub imported: Vec<ImportedCard>,
    /// Card names that couldn't be resolved.
    pub unresolved: Vec<UnresolvedCard>,
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn parse_lines(text: &str) -> Vec<ImportLine> {
        ImportDeckCards::parse(Uuid::nil(), Uuid::nil(), text, false, None, ImportMode::Add).lines
    }

    /// Does an entry (as the user typed it) resolve to a card stored under
    /// `stored_name`? Mirrors the import: the card is indexed by its full name
    /// and front face; the entry is tried as-written then by its front face.
    fn resolves(entry: &str, stored_name: &str) -> bool {
        let index: Vec<String> = std::iter::once(stored_name.to_lowercase())
            .chain(dfc_front_face(stored_name).map(str::to_lowercase))
            .collect();
        let entry = entry.to_lowercase();
        let front = entry_front_face(&entry).to_lowercase();
        index.contains(&entry) || index.contains(&front)
    }

    #[test]
    fn dfc_front_face_extracts_only_double_faced() {
        assert_eq!(
            dfc_front_face("Boggart Trawler // Boggart Bog"),
            Some("Boggart Trawler")
        );
        assert_eq!(dfc_front_face("Lightning Bolt"), None);
    }

    #[test]
    fn entry_front_face_collapses_separators() {
        assert_eq!(entry_front_face("Boggart Trawler // Boggart Bog"), "Boggart Trawler");
        assert_eq!(entry_front_face("Boggart Trawler / Boggart Bog"), "Boggart Trawler");
        assert_eq!(entry_front_face("Boggart Trawler"), "Boggart Trawler");
        assert_eq!(entry_front_face("Lightning Bolt"), "Lightning Bolt");
    }

    #[test]
    fn double_faced_card_resolves_from_every_format() {
        let card = "Boggart Trawler // Boggart Bog";
        for entry in [
            "Boggart Trawler // Boggart Bog", // full name
            "Boggart Trawler / Boggart Bog",  // single-slash spelling
            "Boggart Trawler",                // front face only
            "boggart trawler",                // case-insensitive front
        ] {
            assert!(resolves(entry, card), "entry {entry:?} should resolve to {card:?}");
        }
    }

    #[test]
    fn single_faced_card_still_resolves_exactly() {
        assert!(resolves("Lightning Bolt", "Lightning Bolt"));
        assert!(!resolves("Lightning", "Lightning Bolt"));
    }

    #[test]
    fn back_face_alone_does_not_resolve() {
        // Decklists name double-faced cards by the front (or full name); the back
        // face on its own is intentionally not a match.
        assert!(!resolves("Boggart Bog", "Boggart Trawler // Boggart Bog"));
    }

    #[test]
    fn moxfield_plain_format() {
        let lines = parse_lines("4 Lightning Bolt\n1 Island\nSol Ring");
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].quantity, 4);
        assert_eq!(lines[0].card_name, "Lightning Bolt");
        assert_eq!(lines[1].quantity, 1);
        assert_eq!(lines[1].card_name, "Island");
        assert_eq!(lines[2].quantity, 1);
        assert_eq!(lines[2].card_name, "Sol Ring");
    }

    #[test]
    fn archidekt_format_with_set_and_tags() {
        let lines =
            parse_lines("1x Aether Hub (drc) 145 [Land]\n1x Dr. Madison Li (pip) 531 *F* [Draw]");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].quantity, 1);
        assert_eq!(lines[0].card_name, "Aether Hub");
        assert_eq!(lines[1].quantity, 1);
        assert_eq!(lines[1].card_name, "Dr. Madison Li");
    }

    #[test]
    fn quantity_with_x_suffix() {
        let lines = parse_lines("5x Island (dft) 282 [Land]");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].quantity, 5);
        assert_eq!(lines[0].card_name, "Island");
    }

    #[test]
    fn skips_empty_and_whitespace_lines() {
        let lines = parse_lines("\n  \n4 Lightning Bolt\n\n");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].card_name, "Lightning Bolt");
    }

    #[test]
    fn skips_zero_quantity() {
        let lines = parse_lines("0 Lightning Bolt");
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn strip_metadata_no_parens() {
        assert_eq!(strip_trailing_metadata("Lightning Bolt"), "Lightning Bolt");
    }

    #[test]
    fn strip_metadata_with_parens() {
        assert_eq!(
            strip_trailing_metadata("Aether Hub (drc) 145 [Land]"),
            "Aether Hub"
        );
    }

    #[test]
    fn strip_metadata_with_brackets_only() {
        assert_eq!(strip_trailing_metadata("Aether Hub [Land]"), "Aether Hub");
    }

    #[test]
    fn strip_metadata_foil_marker() {
        assert_eq!(
            strip_trailing_metadata("Dr. Madison Li (pip) 531 *F* [Draw]"),
            "Dr. Madison Li"
        );
    }

    #[test]
    fn sideboard_header_sets_board() {
        let lines = parse_lines("4 Lightning Bolt\n\n// Sideboard\n2 Rest in Peace");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].board, Board::Deck);
        assert_eq!(lines[0].card_name, "Lightning Bolt");
        assert_eq!(lines[1].board, Board::Sideboard);
        assert_eq!(lines[1].card_name, "Rest in Peace");
    }

    #[test]
    fn maybeboard_header_sets_board() {
        let lines = parse_lines("1 Sol Ring\n// Maybeboard\n1 Mana Crypt");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].board, Board::Deck);
        assert_eq!(lines[1].board, Board::Maybeboard);
    }

    #[test]
    fn multiple_sections() {
        let text = "1 Sol Ring\n// Sideboard\n1 Rest in Peace\n// Maybeboard\n1 Mana Crypt";
        let lines = parse_lines(text);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].board, Board::Deck);
        assert_eq!(lines[1].board, Board::Sideboard);
        assert_eq!(lines[2].board, Board::Maybeboard);
    }

    #[test]
    fn header_without_space() {
        let lines = parse_lines("//Sideboard\n1 Rest in Peace");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].board, Board::Sideboard);
    }

    #[test]
    fn default_board_is_deck() {
        let lines = parse_lines("4 Lightning Bolt");
        assert_eq!(lines[0].board, Board::Deck);
    }

    #[test]
    fn board_override_forces_all_lines() {
        let text = "1 Sol Ring\n// Sideboard\n1 Rest in Peace\n// Deck\n1 Lightning Bolt";
        let lines = ImportDeckCards::parse(
            Uuid::nil(),
            Uuid::nil(),
            text,
            false,
            Some(Board::Maybeboard),
            ImportMode::Add,
        )
        .lines;
        assert_eq!(lines.len(), 3);
        assert!(lines.iter().all(|l| l.board == Board::Maybeboard));
    }
}
