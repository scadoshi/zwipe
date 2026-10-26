//! Suggested commanders: the ranked list and its dismissals.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::card::Card;

/// Suggestions returned when the request names no `limit`.
pub const DEFAULT_SUGGESTION_LIMIT: u32 = 10;

/// Most suggestions one request returns; a larger `limit` is clamped to it.
pub const MAX_SUGGESTION_LIMIT: u32 = 100;

/// Most oracle tags [`HttpBuilderProfile::tags`] carries. The app's tag table
/// shows only the first few, and each card role reveals its own tags from the
/// whole list, so it runs deep enough that the top roles find theirs.
pub const PROFILE_TAG_SENT: usize = 100;

/// Card roles [`HttpBuilderProfile::roles`] carries, one bar each.
pub const PROFILE_ROLE_SHOWN: usize = 10;

/// Response of `GET /api/commander/suggestions`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpCommanderSuggestions {
    /// Ranked best first.
    pub suggestions: Vec<HttpCommanderSuggestion>,
    /// Decks with a commander that fed the profile. 0 is the cold start, when
    /// `suggestions` is empty and the client shows its empty state.
    pub deck_count: u32,
    /// What the user builds, summarized from the same decks. `None` at the
    /// cold start, and absent from older servers' responses.
    #[serde(default)]
    pub profile: Option<HttpBuilderProfile>,
}

/// The colors and oracle tags across the user's commander decks, the profile
/// the suggestions are ranked against.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpBuilderProfile {
    /// Decks with a commander, the same count as
    /// [`HttpCommanderSuggestions::deck_count`].
    pub deck_count: u32,
    /// Each color the user's commanders use, with how many decks use it, most
    /// decks first and WUBRG order between equals. Colors no deck uses are
    /// absent.
    pub colors: Vec<HttpColorAffinity>,
    /// The exact color identity the user builds most, with its deck count.
    /// `None` when there is more than one deck and every identity is unique.
    pub top_identity: Option<HttpColorIdentityCount>,
    /// The user's most impactful oracle tags across the profiled decks,
    /// highest [`HttpProfileTag::score`] first and slug between equals, at
    /// most [`PROFILE_TAG_SENT`].
    pub tags: Vec<HttpProfileTag>,
    /// The card roles the profiled decks' tags fall under, most decks first
    /// and role slug between equals, at most [`PROFILE_ROLE_SHOWN`]. Counted
    /// over every tag in the profiles, not only the `tags` shown. Empty from
    /// older servers.
    #[serde(default)]
    pub roles: Vec<HttpProfileRole>,
}

/// A card role the user's decks lean on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpProfileRole {
    /// The card role slug (`CardRole`'s snake_case form).
    pub role: String,
    /// Profiled decks with at least one oracle tag under the role, above 0.
    pub decks: u32,
}

/// How many of the user's commander decks share one exact color identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpColorIdentityCount {
    /// The identity as `W`, `U`, `B`, `R`, `G` in WUBRG order; empty for
    /// colorless.
    pub colors: Vec<String>,
    /// Commander decks whose commander has exactly this identity.
    pub decks: u32,
}

/// An oracle tag the user's decks lean on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpProfileTag {
    /// The oracle tag slug.
    pub slug: String,
    /// The card role slug the tag falls under, as [`HttpSharedTag::role`].
    pub role: Option<String>,
    /// Profiled decks whose profile carries the tag, above 0.
    pub decks: u32,
    /// Cards carrying the tag, summed over those decks, each card counted
    /// once per deck. 0 from older servers.
    #[serde(default)]
    pub cards: u32,
    /// The tag's impact: its share of each profiled deck's tag profile
    /// (cards carrying it times how rare it is across all cards, over the
    /// deck's top tags), summed over the decks. Between 0 and the deck count;
    /// only its ratio to other tags' scores means anything. 0.0 from older
    /// servers.
    #[serde(default)]
    pub score: f64,
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
    /// Oracle tags the commander shares with the user's decks, strongest
    /// first, at most 5.
    pub tags: Vec<HttpSharedTag>,
    /// The commander's colors the user's decks use, most-used first. Colors no
    /// deck uses are absent.
    pub colors: Vec<HttpColorAffinity>,
}

/// An oracle tag the user's decks and the commander both lean on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpSharedTag {
    /// The oracle tag slug.
    pub slug: String,
    /// The card role slug (`CardRole`'s snake_case form) the tag falls under,
    /// the same one the card's own `oracle_tags_by_role` files it beneath.
    /// `None` is the "Other" bucket.
    pub role: Option<String>,
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
            tags: vec![
                HttpSharedTag {
                    slug: "sacrifice-outlet".to_string(),
                    role: Some("sacrifice".to_string()),
                },
                HttpSharedTag {
                    slug: "typal-goblin".to_string(),
                    role: None,
                },
            ],
            colors: vec![HttpColorAffinity {
                color: "R".to_string(),
                decks: 3,
            }],
        };
        let json = serde_json::to_string(&reasons).unwrap();
        assert_eq!(
            json,
            r#"{"tags":[{"slug":"sacrifice-outlet","role":"sacrifice"},{"slug":"typal-goblin","role":null}],"colors":[{"color":"R","decks":3}]}"#
        );
        let back: HttpSuggestionReasons = serde_json::from_str(&json).unwrap();
        assert_eq!(back, reasons);
    }

    #[test]
    fn cold_start_round_trip() {
        let cold = HttpCommanderSuggestions {
            suggestions: vec![],
            deck_count: 0,
            profile: None,
        };
        let json = serde_json::to_string(&cold).unwrap();
        assert_eq!(json, r#"{"suggestions":[],"deck_count":0,"profile":null}"#);
        let back: HttpCommanderSuggestions = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cold);
    }

    #[test]
    fn old_shape_without_profile_parses_to_none() {
        let back: HttpCommanderSuggestions =
            serde_json::from_str(r#"{"suggestions":[],"deck_count":3}"#).unwrap();
        assert_eq!(back.deck_count, 3);
        assert_eq!(back.profile, None);
    }

    #[test]
    fn profile_round_trip() {
        let profile = HttpBuilderProfile {
            deck_count: 7,
            colors: vec![
                HttpColorAffinity {
                    color: "R".to_string(),
                    decks: 5,
                },
                HttpColorAffinity {
                    color: "W".to_string(),
                    decks: 4,
                },
            ],
            top_identity: Some(HttpColorIdentityCount {
                colors: vec!["W".to_string(), "R".to_string()],
                decks: 3,
            }),
            tags: vec![HttpProfileTag {
                slug: "sacrifice-outlet".to_string(),
                role: Some("sacrifice".to_string()),
                decks: 6,
                cards: 23,
                score: 1.5,
            }],
            roles: vec![HttpProfileRole {
                role: "sacrifice".to_string(),
                decks: 6,
            }],
        };
        let json = serde_json::to_string(&profile).unwrap();
        assert_eq!(
            json,
            r#"{"deck_count":7,"colors":[{"color":"R","decks":5},{"color":"W","decks":4}],"top_identity":{"colors":["W","R"],"decks":3},"tags":[{"slug":"sacrifice-outlet","role":"sacrifice","decks":6,"cards":23,"score":1.5}],"roles":[{"role":"sacrifice","decks":6}]}"#
        );
        let back: HttpBuilderProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, profile);

        let no_top = r#"{"deck_count":2,"colors":[],"top_identity":null,"tags":[]}"#;
        let back: HttpBuilderProfile = serde_json::from_str(no_top).unwrap();
        assert_eq!(back.top_identity, None);
        assert_eq!(back.roles, vec![], "old shape without roles parses");
    }

    #[test]
    fn old_tag_shape_without_cards_or_score_parses_to_zero() {
        let back: HttpProfileTag =
            serde_json::from_str(r#"{"slug":"treasure","role":null,"decks":3}"#).unwrap();
        assert_eq!(
            back,
            HttpProfileTag {
                slug: "treasure".to_string(),
                role: None,
                decks: 3,
                cards: 0,
                score: 0.0,
            }
        );
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
