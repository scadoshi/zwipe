//! Suggested commanders: the ranked list and its dismissals.

use chrono::NaiveDate;
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

/// Most staples [`HttpBuilderProfile::staples`] carries.
pub const PROFILE_STAPLES_SENT: usize = 100;

/// Fewest of the user's decks a card must be in to count as a staple.
pub const STAPLE_MIN_DECKS: u32 = 2;

/// Response of `GET /api/deck/playstyle`: the user's deck count and color
/// map, a light read for Home that builds no tag profiles and scores no
/// commanders.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpPlayStyleSummary {
    /// Decks in any format with a commander or a mainboard card, the same
    /// count as [`HttpCommanderSuggestions::deck_count`]. 0 is the cold start.
    pub deck_count: u32,
    /// The same color map as [`HttpBuilderProfile::color_map`], summed from
    /// the same decks, so Home draws the shape the Playstyle screen does.
    /// Every color and pair is present, zeros included. `None` from older
    /// servers.
    #[serde(default)]
    pub color_map: Option<HttpColorMap>,
}

/// Response of `GET /api/commander/suggestions`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpCommanderSuggestions {
    /// Ranked best first.
    pub suggestions: Vec<HttpCommanderSuggestion>,
    /// Decks in any format that fed the profile. 0 is the cold start, when
    /// `suggestions` is empty and the client shows its empty state.
    pub deck_count: u32,
    /// What the user builds, summarized from the same decks. `None` at the
    /// cold start, and absent from older servers' responses.
    #[serde(default)]
    pub profile: Option<HttpBuilderProfile>,
}

/// The colors and oracle tags across the user's decks in every format, the
/// profile
/// the suggestions are ranked against.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpBuilderProfile {
    /// Decks summarized, the same count as
    /// [`HttpCommanderSuggestions::deck_count`].
    pub deck_count: u32,
    /// Each color the user's decks use, with how many decks use it, most
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
    /// The cards on the most profiled decks' mainboards, at least
    /// [`STAPLE_MIN_DECKS`] each, most decks first, then name, then oracle
    /// id, at most [`PROFILE_STAPLES_SENT`]. Basic lands are left out. Empty
    /// when no card repeats, and from older servers.
    #[serde(default)]
    pub staples: Vec<HttpStaple>,
    /// How the user's cards and decks spread over the five colors and the
    /// ten color pairs. `None` from older servers.
    #[serde(default)]
    pub color_map: Option<HttpColorMap>,
}

/// The cards and decks behind the color map, summed across the user's decks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpColorMap {
    /// All five colors in WUBRG order, each with the mainboard and command
    /// zone cards whose color identity includes it. A card counts once per deck it is in, so
    /// a two-color card counts toward both its colors.
    pub colors: Vec<HttpColorCards>,
    /// Mainboard and command zone cards with an empty color identity,
    /// counted the same way.
    pub colorless_cards: u32,
    /// All ten color pairs, each with the decks whose color identity
    /// includes both colors, in WUBRG order (`WU`, `WB`, `WR`, `WG`, `UB`,
    /// `UR`, `UG`, `BR`, `BG`, `RG`). A deck of three or more colors counts
    /// toward every pair it holds.
    pub pairs: Vec<HttpColorPair>,
}

/// One color's cards in the color map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpColorCards {
    /// One of `W`, `U`, `B`, `R`, `G`.
    pub color: String,
    /// Mainboard and command zone cards whose color identity includes the
    /// color, 0 or more.
    pub cards: u32,
}

/// One color pair's decks in the color map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpColorPair {
    /// The two colors in WUBRG order.
    pub colors: [String; 2],
    /// Decks whose color identity includes both colors, 0 or more.
    pub decks: u32,
}

/// A card on several of the user's decks' mainboards.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpStaple {
    /// The printing on the most recently updated deck that has the card.
    pub card: Card,
    /// Profiled decks with the card on their mainboard, counted once per
    /// deck however many copies it runs, at least [`STAPLE_MIN_DECKS`].
    pub decks: u32,
}

/// A card role the user's decks lean on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpProfileRole {
    /// The card role slug (`CardRole`'s snake_case form).
    pub role: String,
    /// Profiled decks with at least one oracle tag under the role, above 0.
    pub decks: u32,
}

/// How many of the user's decks share one exact color identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpColorIdentityCount {
    /// The identity as `W`, `U`, `B`, `R`, `G` in WUBRG order; empty for
    /// colorless.
    pub colors: Vec<String>,
    /// Decks with exactly these colors: the commander's identity when the
    /// deck has one, otherwise the union of its mainboard cards' identities.
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
    /// (cards carrying it times the log of how much more the deck leans on it
    /// than all cards do, over the deck's top tags; see
    /// [`crate::domain::deck::tag_profile`]), summed over the decks. Between
    /// 0 and the deck count;
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

/// How many of the user's decks use one of the commander's colors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpColorAffinity {
    /// One of `W`, `U`, `B`, `R`, `G`.
    pub color: String,
    /// Decks whose colors include it, above 0.
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
            staples: vec![],
            color_map: None,
        };
        let json = serde_json::to_string(&profile).unwrap();
        assert_eq!(
            json,
            r#"{"deck_count":7,"colors":[{"color":"R","decks":5},{"color":"W","decks":4}],"top_identity":{"colors":["W","R"],"decks":3},"tags":[{"slug":"sacrifice-outlet","role":"sacrifice","decks":6,"cards":23,"score":1.5}],"roles":[{"role":"sacrifice","decks":6}],"staples":[],"color_map":null}"#
        );
        let back: HttpBuilderProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, profile);

        let no_top = r#"{"deck_count":2,"colors":[],"top_identity":null,"tags":[]}"#;
        let back: HttpBuilderProfile = serde_json::from_str(no_top).unwrap();
        assert_eq!(back.top_identity, None);
        assert_eq!(back.roles, vec![], "old shape without roles parses");
        assert_eq!(back.staples, vec![], "old shape without staples parses");
        assert_eq!(back.color_map, None, "old shape without a color map parses");
    }

    #[test]
    fn color_map_round_trip() {
        let map = HttpColorMap {
            colors: vec![
                HttpColorCards {
                    color: "W".to_string(),
                    cards: 12,
                },
                HttpColorCards {
                    color: "R".to_string(),
                    cards: 0,
                },
            ],
            colorless_cards: 4,
            pairs: vec![HttpColorPair {
                colors: ["W".to_string(), "R".to_string()],
                decks: 2,
            }],
        };
        let json = serde_json::to_string(&map).unwrap();
        assert_eq!(
            json,
            r#"{"colors":[{"color":"W","cards":12},{"color":"R","cards":0}],"colorless_cards":4,"pairs":[{"colors":["W","R"],"decks":2}]}"#
        );
        let back: HttpColorMap = serde_json::from_str(&json).unwrap();
        assert_eq!(back, map);

        let profile = format!(
            r#"{{"deck_count":2,"colors":[],"top_identity":null,"tags":[],"roles":[],"color_map":{json}}}"#
        );
        let back: HttpBuilderProfile = serde_json::from_str(&profile).unwrap();
        assert_eq!(back.color_map, Some(map), "a profile carries its color map");
    }

    #[test]
    fn staples_round_trip() {
        let profile = HttpBuilderProfile {
            deck_count: 3,
            colors: vec![],
            top_identity: None,
            tags: vec![],
            roles: vec![],
            staples: vec![
                HttpStaple {
                    card: crate::test_utils::make_card("Sol Ring"),
                    decks: 3,
                },
                HttpStaple {
                    card: crate::test_utils::make_card("Arcane Signet"),
                    decks: 2,
                },
            ],
            color_map: None,
        };
        let json = serde_json::to_string(&profile).unwrap();
        let back: HttpBuilderProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, profile);
        assert!(json.contains(r#""decks":3}"#), "{json}");
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
    fn play_style_summary_round_trip() {
        let summary = HttpPlayStyleSummary {
            deck_count: 5,
            color_map: Some(HttpColorMap {
                colors: vec![HttpColorCards {
                    color: "U".to_string(),
                    cards: 30,
                }],
                colorless_cards: 2,
                pairs: vec![],
            }),
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert_eq!(
            json,
            r#"{"deck_count":5,"color_map":{"colors":[{"color":"U","cards":30}],"colorless_cards":2,"pairs":[]}}"#
        );
        let back: HttpPlayStyleSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back, summary);

        let old: HttpPlayStyleSummary =
            serde_json::from_str(r#"{"deck_count":3,"identities":[]}"#).unwrap();
        assert_eq!(old.color_map, None, "an older server's summary parses");
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

/// Days [`HttpActivity::days`] reaches back, counting the last one: 53
/// weeks, so the heatmap fills its first column whatever the weekday.
pub const ACTIVITY_DAYS: i64 = 371;

/// Card roles [`HttpActivity::roles`] carries, the most seen first.
pub const ACTIVITY_ROLES_SENT: usize = 10;

/// A role or tag needs this many dealt cards (added, skipped and saved
/// together) to be sent, so one seen twice does not read 100%.
pub const ACTIVITY_MIN_SEEN: u64 = 10;

/// Oracle tags [`HttpActivity::tags`] carries, the most seen first.
pub const ACTIVITY_TAGS_SENT: usize = 100;

/// Most cards [`HttpActivity::passes`] carries.
pub const ACTIVITY_PASSES_SENT: usize = 10;

/// What the user did in the app and with the cards it dealt them: daily
/// swipes and searches over the last [`ACTIVITY_DAYS`], what the swipes
/// came to, the keep rate per card role, and the cards they pass on most.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpActivity {
    /// The server's UTC date, the last day of the heatmap.
    pub through: NaiveDate,
    /// Days with any swipe or search from `through` back over
    /// [`ACTIVITY_DAYS`], oldest first. A day with none is left out.
    pub days: Vec<HttpActivityDay>,
    /// Every card swiped on, across all of the user's decks.
    pub outcomes: HttpSwipeOutcomes,
    /// What dealt cards came to per card role, in decks with a commander, at
    /// most [`ACTIVITY_ROLES_SENT`], the most seen first.
    pub roles: Vec<HttpRoleRates>,
    /// What dealt cards came to per oracle tag under a card role, in decks
    /// with a commander, with every other player's counts for the same tag,
    /// at most [`ACTIVITY_TAGS_SENT`], the most seen first.
    pub tags: Vec<HttpTagRates>,
    /// The cards skipped most and never kept or saved, in decks with a
    /// commander, at most [`ACTIVITY_PASSES_SENT`], the most skipped first.
    pub passes: Vec<HttpHardPass>,
}

/// One day's activity, by the server's UTC calendar.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpActivityDay {
    /// The UTC date.
    pub day: NaiveDate,
    /// Swipes in any direction on any screen.
    pub swipes: u32,
    /// Searches run.
    pub searches: u32,
}

/// What the user's swipes on dealt cards came to.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpSwipeOutcomes {
    /// Added to a deck.
    pub added: u64,
    /// Skipped.
    pub skipped: u64,
    /// Saved to a maybeboard.
    pub maybed: u64,
    /// Taken back out of a deck.
    pub removed: u64,
}

/// What one card role's dealt cards came to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpRoleRates {
    /// The card role slug (`CardRole`'s snake_case form).
    pub role: String,
    /// Its cards' outcomes.
    pub outcomes: HttpSwipeOutcomes,
}

/// What one oracle tag's dealt cards came to, for the user and for every
/// other player.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpTagRates {
    /// The oracle tag slug.
    pub tag: String,
    /// The card role slug it falls under, the first by name when several.
    pub role: String,
    /// The user's outcomes on cards with the tag.
    pub outcomes: HttpSwipeOutcomes,
    /// Every other player's outcomes on cards with the tag.
    pub everyone: HttpSwipeOutcomes,
}

/// A card the user keeps passing on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpHardPass {
    /// The card's most recent normal printing.
    pub card: Card,
    /// Times skipped, across every commander.
    pub skipped: u32,
    /// The names of the commanders it was skipped under, by name.
    pub commanders: Vec<String>,
}
