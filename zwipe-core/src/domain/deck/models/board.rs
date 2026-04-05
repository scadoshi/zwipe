//! Board classification for deck cards.
//!
//! A card in a deck belongs to one of three boards: the active deck,
//! the maybeboard (considering), or the sideboard (tournament swap pool).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error returned when parsing an invalid board string.
#[derive(Debug, Clone, Error)]
#[error("invalid board")]
pub struct InvalidBoard;

/// Which board a deck card belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Board {
    /// Active deck — counted in metrics, validated against format rules.
    #[default]
    Deck,
    /// Considering — excluded from metrics and validation.
    Maybeboard,
    /// Tournament sideboard — excluded from metrics, validated separately.
    Sideboard,
}

impl Board {
    /// Whether this card is in the active deck.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Deck)
    }

    /// Whether this card is on the maybeboard.
    pub fn is_maybeboard(&self) -> bool {
        matches!(self, Self::Maybeboard)
    }

    /// Whether this card is in the sideboard.
    pub fn is_sideboard(&self) -> bool {
        matches!(self, Self::Sideboard)
    }

    /// Lowercase name matching the database TEXT column value.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Deck => "deck",
            Self::Maybeboard => "maybeboard",
            Self::Sideboard => "sideboard",
        }
    }
}

impl TryFrom<&str> for Board {
    type Error = InvalidBoard;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "deck" => Ok(Self::Deck),
            "maybeboard" => Ok(Self::Maybeboard),
            "sideboard" => Ok(Self::Sideboard),
            _ => Err(InvalidBoard),
        }
    }
}

impl TryFrom<String> for Board {
    type Error = InvalidBoard;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_deck() {
        assert_eq!(Board::default(), Board::Deck);
    }

    #[test]
    fn round_trip_through_str() {
        for board in [Board::Deck, Board::Maybeboard, Board::Sideboard] {
            let name = board.display_name();
            let parsed = Board::try_from(name).unwrap();
            assert_eq!(board, parsed);
        }
    }

    #[test]
    fn round_trip_through_serde() {
        for board in [Board::Deck, Board::Maybeboard, Board::Sideboard] {
            let json = serde_json::to_string(&board).unwrap();
            let parsed: Board = serde_json::from_str(&json).unwrap();
            assert_eq!(board, parsed);
        }
    }

    #[test]
    fn invalid_board_rejected() {
        assert!(Board::try_from("notaboard").is_err());
    }

    #[test]
    fn predicates() {
        assert!(Board::Deck.is_active());
        assert!(!Board::Deck.is_maybeboard());
        assert!(!Board::Deck.is_sideboard());

        assert!(!Board::Maybeboard.is_active());
        assert!(Board::Maybeboard.is_maybeboard());
        assert!(!Board::Maybeboard.is_sideboard());

        assert!(!Board::Sideboard.is_active());
        assert!(!Board::Sideboard.is_maybeboard());
        assert!(Board::Sideboard.is_sideboard());
    }
}
