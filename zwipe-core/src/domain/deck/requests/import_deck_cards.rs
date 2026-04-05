//! Deck card import operation.
//!
//! Parses plain-text decklists (`[qty] [card name]` per line),
//! resolves card names against the database, and bulk-inserts matched cards.
//! Supports `// Sideboard` and `// Maybeboard` section headers.

use crate::domain::deck::Board;
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
}

impl ImportDeckCards {
    /// Parses plain text into an import request.
    ///
    /// Format: `[qty] [card name]` per line.
    /// Quantity defaults to 1 if the first word is not a number.
    /// Empty and whitespace-only lines are skipped.
    pub fn parse(user_id: Uuid, deck_id: Uuid, text: &str, email_verified: bool) -> Self {
        let mut current_board = Board::Deck;
        let mut lines = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Detect section headers: "// Sideboard", "// Maybeboard", "//Sideboard", etc.
            let header = trimmed.strip_prefix("//").map(|s| s.trim().to_lowercase());
            if let Some(ref h) = header {
                match h.as_str() {
                    "sideboard" => { current_board = Board::Sideboard; continue; }
                    "maybeboard" => { current_board = Board::Maybeboard; continue; }
                    "deck" => { current_board = Board::Deck; continue; }
                    // Skip other comment headers like "// Commander"
                    _ if !h.is_empty() => continue,
                    _ => {}
                }
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
        ImportDeckCards::parse(Uuid::nil(), Uuid::nil(), text, false).lines
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
        let lines = parse_lines("1x Aether Hub (drc) 145 [Land]\n1x Dr. Madison Li (pip) 531 *F* [Draw]");
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
        assert_eq!(
            strip_trailing_metadata("Aether Hub [Land]"),
            "Aether Hub"
        );
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
}
