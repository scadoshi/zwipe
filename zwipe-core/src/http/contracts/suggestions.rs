//! Suggested commanders: the ranked list and its dismissals.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::card::Card;

/// Response body for `GET /api/commander/suggestions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpCommanderSuggestions {
    /// Ranked best first, at most the requested limit.
    pub suggestions: Vec<HttpCommanderSuggestion>,
    /// Decks with a commander that fed the profile; 0 means cold start.
    pub deck_count: u32,
}

/// One suggested commander with the score terms behind it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpCommanderSuggestion {
    /// The commander, in its preferred printing.
    pub card: Card,
    /// Why it ranked where it did.
    pub reasons: HttpSuggestionReasons,
}

/// The score terms a client turns into plain lines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpSuggestionReasons {
    /// Oracle tags shared with the user's decks, strongest first, at most 3.
    pub themes: Vec<HttpSharedTheme>,
    /// Only colors with `decks > 0`, most decks first.
    pub colors: Vec<HttpColorAffinity>,
}

/// An oracle tag the commander shares with the user's decks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpSharedTheme {
    /// Oracle tag slug.
    pub slug: String,
    /// Oracle tag display label.
    pub label: String,
}

/// How many of the user's decks share one of the commander's colors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpColorAffinity {
    /// One of `W`, `U`, `B`, `R`, `G`.
    pub color: String,
    /// The user's commander decks that include this color.
    pub decks: u32,
}

/// Request body for `POST /api/commander/suggestions/dismissals`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpDismissCommander {
    /// Oracle id of the commander to hide for a week.
    pub oracle_id: Uuid,
}

/// Response body for `DELETE /api/commander/suggestions/dismissals`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpClearedDismissals {
    /// Dismissal rows removed.
    pub cleared: u32,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_utils::make_card;

    #[test]
    fn suggestions_round_trip() {
        let body = HttpCommanderSuggestions {
            suggestions: vec![HttpCommanderSuggestion {
                card: make_card("Krenko, Mob Boss"),
                reasons: HttpSuggestionReasons {
                    themes: vec![
                        HttpSharedTheme {
                            slug: "tokens".into(),
                            label: "Tokens".into(),
                        },
                        HttpSharedTheme {
                            slug: "goblin-tribal".into(),
                            label: "Goblins".into(),
                        },
                    ],
                    colors: vec![HttpColorAffinity {
                        color: "R".into(),
                        decks: 3,
                    }],
                },
            }],
            deck_count: 4,
        };
        let json = serde_json::to_string(&body).unwrap();
        let back: HttpCommanderSuggestions = serde_json::from_str(&json).unwrap();
        assert_eq!(back, body);
    }

    #[test]
    fn cold_start_is_an_empty_list_with_zero_decks() {
        let json = r#"{"suggestions":[],"deck_count":0}"#;
        let body: HttpCommanderSuggestions = serde_json::from_str(json).unwrap();
        assert!(body.suggestions.is_empty());
        assert_eq!(body.deck_count, 0);
    }

    #[test]
    fn dismissal_bodies_round_trip() {
        let dismiss = HttpDismissCommander {
            oracle_id: Uuid::from_u128(7),
        };
        let json = serde_json::to_string(&dismiss).unwrap();
        assert_eq!(
            json,
            r#"{"oracle_id":"00000000-0000-0000-0000-000000000007"}"#
        );
        let back: HttpDismissCommander = serde_json::from_str(&json).unwrap();
        assert_eq!(back, dismiss);

        let cleared: HttpClearedDismissals = serde_json::from_str(r#"{"cleared":12}"#).unwrap();
        assert_eq!(cleared, HttpClearedDismissals { cleared: 12 });
    }
}
