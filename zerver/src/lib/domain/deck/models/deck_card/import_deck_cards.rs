//! Deck card import operation.
//!
//! Parses plain-text decklists (`[qty] [card name]` per line),
//! resolves card names against the database, and bulk-inserts matched cards.

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use serde::{Deserialize, Serialize};
#[cfg(feature = "zerver")]
use thiserror::Error;
use uuid::Uuid;

/// A parsed line from import text.
#[derive(Debug, Clone)]
pub struct ImportLine {
    /// Number of copies.
    pub quantity: i32,
    /// Card name (as entered by the user).
    pub card_name: String,
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
}

impl ImportDeckCards {
    /// Parses plain text into an import request.
    ///
    /// Format: `[qty] [card name]` per line.
    /// Quantity defaults to 1 if the first word is not a number.
    /// Empty and whitespace-only lines are skipped.
    pub fn parse(user_id: Uuid, deck_id: Uuid, text: &str) -> Self {
        let lines = text
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return None;
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
                    return None;
                }
                Some(ImportLine {
                    quantity,
                    card_name,
                })
            })
            .collect();
        Self {
            user_id,
            deck_id,
            lines,
        }
    }
}

/// Strips trailing metadata from a card name.
///
/// Handles formats like:
/// - Moxfield: `"Aether Hub"` (no metadata)
/// - Archidekt: `"Aether Hub (drc) 145 [Land]"` → `"Aether Hub"`
/// - Archidekt foil: `"Dr. Madison Li (pip) 531 *F* [Draw]"` → `"Dr. Madison Li"`
///
/// Strips everything from the first `(` onward, then trims.
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

/// Errors that can occur during deck card import.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ImportDeckCardsError {
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
    /// Deck not found or inaccessible.
    #[error(transparent)]
    DeckNotFound(#[from] GetDeckProfileError),
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn parse_lines(text: &str) -> Vec<ImportLine> {
        ImportDeckCards::parse(Uuid::nil(), Uuid::nil(), text).lines
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
}
