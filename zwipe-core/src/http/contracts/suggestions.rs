//! Suggested commanders: the ranked list and its dismissals.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::card::Card;

/// Suggestions returned when the request names no `limit`.
pub const DEFAULT_SUGGESTION_LIMIT: u32 = 10;

/// Most suggestions one request returns; a larger `limit` is clamped to it.
pub const MAX_SUGGESTION_LIMIT: u32 = 100;

/// Response of `GET /api/commander/suggestions`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpCommanderSuggestions {
    /// Ranked best first.
    pub suggestions: Vec<HttpCommanderSuggestion>,
    /// Decks with a commander that fed the profile. 0 is the cold start, when
    /// `suggestions` is empty and the client shows its empty state.
    pub deck_count: u32,
}

/// One suggested commander and why it fits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpCommanderSuggestion {
    /// The commander's current printing, the shape deck search returns.
    pub card: Card,
    /// The score terms behind the suggestion.
    pub reasons: HttpSuggestionReasons,
}

/// The score terms behind one suggestion, for the client to word.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpSuggestionReasons {
    /// Themes the commander shares with the user's decks, strongest first, at
    /// most 3.
    pub themes: Vec<HttpSharedTheme>,
    /// The commander's colors the user's decks use, most-used first. Colors no
    /// deck uses are absent.
    pub colors: Vec<HttpColorAffinity>,
}

/// An oracle tag the user's decks and the commander both lean on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpSharedTheme {
    /// The oracle tag slug.
    pub slug: String,
    /// The tag's human label; the slug when the catalog has none.
    pub label: String,
}

/// How many of the user's commander decks use one of the commander's colors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpColorAffinity {
    /// One of `W`, `U`, `B`, `R`, `G`.
    pub color: String,
    /// Commander decks whose commander's color identity includes it, above 0.
    pub decks: u32,
}

/// Body of `POST /api/commander/suggestions/dismissals`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpDismissCommander {
    /// Oracle id of the commander to hide for a week.
    pub oracle_id: Uuid,
}

/// Response of `DELETE /api/commander/suggestions/dismissals`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpClearedDismissals {
    /// Dismissals removed.
    pub cleared: u32,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn reasons_round_trip() {
        let reasons = HttpSuggestionReasons {
            themes: vec![HttpSharedTheme {
                slug: "typal-goblin".to_string(),
                label: "Goblin typal".to_string(),
            }],
            colors: vec![HttpColorAffinity {
                color: "R".to_string(),
                decks: 3,
            }],
        };
        let json = serde_json::to_string(&reasons).unwrap();
        assert_eq!(
            json,
            r#"{"themes":[{"slug":"typal-goblin","label":"Goblin typal"}],"colors":[{"color":"R","decks":3}]}"#
        );
        let back: HttpSuggestionReasons = serde_json::from_str(&json).unwrap();
        assert_eq!(back, reasons);
    }

    #[test]
    fn cold_start_round_trip() {
        let cold = HttpCommanderSuggestions {
            suggestions: vec![],
            deck_count: 0,
        };
        let json = serde_json::to_string(&cold).unwrap();
        assert_eq!(json, r#"{"suggestions":[],"deck_count":0}"#);
        let back: HttpCommanderSuggestions = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cold);
    }

    #[test]
    fn dismissal_bodies_round_trip() {
        let dismiss = HttpDismissCommander {
            oracle_id: Uuid::from_u128(7),
        };
        let back: HttpDismissCommander =
            serde_json::from_str(&serde_json::to_string(&dismiss).unwrap()).unwrap();
        assert_eq!(back, dismiss);

        let cleared: HttpClearedDismissals = serde_json::from_str(r#"{"cleared":4}"#).unwrap();
        assert_eq!(cleared, HttpClearedDismissals { cleared: 4 });
    }
}
