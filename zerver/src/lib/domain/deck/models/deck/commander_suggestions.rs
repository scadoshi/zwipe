//! Suggested commanders: the user's commander decks scored against the
//! precomputed commander tag profiles, the variety cap over the ranking, and
//! the dismissals that hide one.

use std::cmp::Reverse;
use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::suggestions::PROFILE_TAG_SHOWN;

/// Decks whose tag profile a candidate is scored against: the user's most
/// recently updated ones. Each profile is one query, so this bounds the work
/// for a user with many decks; colors are read from every deck regardless.
pub const MAX_PROFILED_DECKS: usize = 12;

/// Scored candidates the service pulls before capping, as many as the largest
/// request, so a short list has candidates to move up once some are capped.
pub const SCORED_POOL: i64 = 100;

/// Most commanders of one exact color identity the ranking keeps in place
/// before the rest of that identity fall behind every other candidate. A user
/// whose decks share one strong theme would otherwise see ten commanders of
/// the same identity; four leaves over half of a ten-row list to the user's
/// other colors while still letting a favorite identity dominate.
pub const MAX_PER_COLOR_IDENTITY: usize = 4;

/// Errors while building suggestions or changing dismissals.
#[derive(Debug, Error)]
pub enum CommanderSuggestionsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

/// Errors while constructing a [`CommanderDismissal`] from a path segment.
#[derive(Debug, Error)]
pub enum InvalidCommanderDismissal {
    /// Invalid oracle ID format.
    #[error(transparent)]
    OracleId(uuid::Error),
}

/// One user's dismissal of one commander.
#[derive(Debug, Clone)]
pub struct CommanderDismissal {
    /// The dismissing user.
    pub user_id: Uuid,
    /// Oracle id of the commander, covering every printing.
    pub oracle_id: Uuid,
}

impl CommanderDismissal {
    /// Creates a dismissal request.
    pub fn new(user_id: Uuid, oracle_id: Uuid) -> Self {
        Self { user_id, oracle_id }
    }

    /// Creates a dismissal request parsing the oracle id from a path segment.
    pub fn from_path(user_id: Uuid, oracle_id: &str) -> Result<Self, InvalidCommanderDismissal> {
        let oracle_id =
            Uuid::try_parse(oracle_id.trim()).map_err(InvalidCommanderDismissal::OracleId)?;
        Ok(Self::new(user_id, oracle_id))
    }
}

/// One of the user's decks that has a commander, as the profile reads it.
#[derive(Debug, Clone)]
pub struct CommanderDeck {
    /// The commander's oracle id.
    pub commander_oracle_id: Uuid,
    /// The commander's color identity as WUBRG short names.
    pub commander_colors: Vec<String>,
    /// Oracle ids of the deck's mainboard cards.
    pub mainboard_oracle_ids: Vec<Uuid>,
}

/// An oracle tag the user's decks and a commander both lean on.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SharedTag {
    /// The oracle tag slug.
    pub slug: String,
    /// The card role the tag falls under, as the card display groups it; `None`
    /// for a tag under no role.
    pub role: Option<String>,
}

/// How many of the user's commander decks use one of a commander's colors.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorAffinity {
    /// One of `W`, `U`, `B`, `R`, `G`.
    pub color: String,
    /// Commander decks whose commander's color identity includes it.
    pub decks: u32,
}

/// A candidate commander as the scoring query ranks it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredCommander {
    /// The commander's oracle id.
    pub oracle_id: Uuid,
    /// The commander's color identity as WUBRG short names.
    pub color_identity: Vec<String>,
    /// The strongest shared tags with the user's best-matching deck, strongest
    /// first.
    pub tags: Vec<SharedTag>,
}

/// Reorders a ranking so that at most `cap` commanders of each exact color
/// identity keep their place; every further commander of that identity moves
/// behind the whole list, in its original order. Nothing is dropped, and
/// commanders that stay keep their relative order, so a stable ranking stays
/// stable.
#[must_use]
pub fn cap_color_identities(scored: Vec<ScoredCommander>, cap: usize) -> Vec<ScoredCommander> {
    let mut kept_per_identity: HashMap<Vec<String>, usize> = HashMap::new();
    let mut kept = Vec::with_capacity(scored.len());
    let mut capped = Vec::new();
    for commander in scored {
        let mut identity = commander.color_identity.clone();
        identity.sort_unstable();
        identity.dedup();
        let seen = kept_per_identity.entry(identity).or_default();
        if *seen < cap {
            *seen += 1;
            kept.push(commander);
        } else {
            capped.push(commander);
        }
    }
    kept.extend(capped);
    kept
}

/// One suggestion with the terms behind it.
#[derive(Debug, Clone)]
pub struct CommanderSuggestion {
    /// The commander's current printing.
    pub card: Card,
    /// Shared oracle tags, strongest first.
    pub tags: Vec<SharedTag>,
    /// The commander's colors the user's decks use, most-used first.
    pub colors: Vec<ColorAffinity>,
}

/// How many of the user's commander decks share one exact color identity.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorIdentityCount {
    /// The identity in WUBRG order; empty for colorless.
    pub colors: Vec<String>,
    /// Decks whose commander has exactly this identity.
    pub decks: u32,
}

/// An oracle tag the user's decks lean on, with how many decks carry it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileTag {
    /// The oracle tag slug.
    pub slug: String,
    /// The card role the tag falls under, as [`SharedTag::role`].
    pub role: Option<String>,
    /// Profiled decks whose profile carries the tag.
    pub decks: u32,
}

/// What the user builds, summarized from the decks the suggestions are
/// ranked against.
#[derive(Debug, Clone, PartialEq)]
pub struct BuilderProfile {
    /// Decks with a commander.
    pub deck_count: u32,
    /// Each color the commanders use with its deck count, most decks first,
    /// WUBRG order between equals.
    pub colors: Vec<ColorAffinity>,
    /// The most-built exact identity; `None` when every identity is unique
    /// across more than one deck.
    pub top_identity: Option<ColorIdentityCount>,
    /// The strongest oracle tags, strongest first, at most
    /// [`PROFILE_TAG_SHOWN`].
    pub tags: Vec<ProfileTag>,
}

/// The WUBRG colors in order.
const WUBRG: [&str; 5] = ["W", "U", "B", "R", "G"];

/// A color's place in WUBRG order; colors outside it sort last.
fn wubrg_index(color: &str) -> usize {
    WUBRG
        .iter()
        .position(|c| *c == color)
        .unwrap_or(WUBRG.len())
}

/// A deck's color identity deduplicated and in WUBRG order.
fn sorted_identity(colors: &[String]) -> Vec<String> {
    let mut identity: Vec<String> = colors.to_vec();
    identity.sort_by_key(|c| wubrg_index(c));
    identity.dedup();
    identity
}

/// Summarizes the user's commander decks: per color the decks using it,
/// the most-built exact identity, and the strongest oracle tags across the
/// deck profiles. `profiles` pairs with `decks` by position (shorter is fine;
/// the rest count as unprofiled). Tag roles are left `None` for the caller to
/// resolve. `None` when there are no decks.
///
/// A tag's strength is the sum of its weight over the profiles that carry it;
/// weights are normalized per deck, so the sum is deck-weighted importance.
/// The top identity breaks ties by more colors, then WUBRG order; it is
/// `None` when more than one deck exists and no identity repeats.
#[must_use]
pub fn summarize_profile(
    decks: &[CommanderDeck],
    profiles: &[Option<serde_json::Value>],
) -> Option<BuilderProfile> {
    if decks.is_empty() {
        return None;
    }
    let deck_count = u32::try_from(decks.len()).unwrap_or(u32::MAX);

    let mut color_decks: HashMap<String, u32> = HashMap::new();
    let mut identity_decks: HashMap<Vec<String>, u32> = HashMap::new();
    for deck in decks {
        let identity = sorted_identity(&deck.commander_colors);
        for color in &identity {
            *color_decks.entry(color.clone()).or_default() += 1;
        }
        *identity_decks.entry(identity).or_default() += 1;
    }
    let mut colors: Vec<ColorAffinity> = color_decks
        .into_iter()
        .filter(|(color, decks)| *decks > 0 && WUBRG.contains(&color.as_str()))
        .map(|(color, decks)| ColorAffinity { color, decks })
        .collect();
    colors.sort_by_key(|c| (Reverse(c.decks), wubrg_index(&c.color)));

    let top_identity = identity_decks
        .into_iter()
        .max_by_key(|(identity, count)| {
            let order: Vec<Reverse<usize>> =
                identity.iter().map(|c| Reverse(wubrg_index(c))).collect();
            (*count, identity.len(), order)
        })
        .filter(|(_, count)| decks.len() == 1 || *count > 1)
        .map(|(colors, decks)| ColorIdentityCount { colors, decks });

    let mut tag_strength: HashMap<String, (f64, u32)> = HashMap::new();
    for profile in profiles.iter().flatten() {
        let Some(weights) = profile.as_object() else {
            continue;
        };
        for (slug, weight) in weights {
            let weight = weight.as_f64().unwrap_or_default();
            let entry = tag_strength.entry(slug.clone()).or_default();
            entry.0 += weight;
            entry.1 += 1;
        }
    }
    let mut ranked: Vec<(String, f64, u32)> = tag_strength
        .into_iter()
        .map(|(slug, (strength, decks))| (slug, strength, decks))
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let tags = ranked
        .into_iter()
        .take(PROFILE_TAG_SHOWN)
        .map(|(slug, _, decks)| ProfileTag {
            slug,
            role: None,
            decks,
        })
        .collect();

    Some(BuilderProfile {
        deck_count,
        colors,
        top_identity,
        tags,
    })
}

/// The ranked suggestions and the deck count that fed them.
#[derive(Debug, Clone)]
pub struct CommanderSuggestions {
    /// Ranked best first.
    pub suggestions: Vec<CommanderSuggestion>,
    /// The user's decks with a commander. 0 means no profile could be built.
    pub deck_count: u32,
    /// The decks summarized; `None` at the cold start.
    pub profile: Option<BuilderProfile>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scored(n: u128, identity: &[&str]) -> ScoredCommander {
        ScoredCommander {
            oracle_id: Uuid::from_u128(n),
            color_identity: identity.iter().map(|c| c.to_string()).collect(),
            tags: vec![],
        }
    }

    fn ids(scored: &[ScoredCommander]) -> Vec<u128> {
        scored.iter().map(|s| s.oracle_id.as_u128()).collect()
    }

    #[test]
    fn empty_stays_empty() {
        assert!(cap_color_identities(vec![], 4).is_empty());
    }

    #[test]
    fn under_the_cap_keeps_the_order() {
        let input = vec![scored(1, &["U"]), scored(2, &["U"]), scored(3, &["R", "W"])];
        let out = cap_color_identities(input.clone(), 4);
        assert_eq!(out, input);
    }

    #[test]
    fn exactly_the_cap_keeps_the_order() {
        let input: Vec<_> = (1..=4).map(|n| scored(n, &["U"])).collect();
        let out = cap_color_identities(input.clone(), 4);
        assert_eq!(out, input);
    }

    #[test]
    fn over_the_cap_moves_the_rest_behind_everything_in_score_order() {
        let input = vec![
            scored(1, &["U"]),
            scored(2, &["U"]),
            scored(3, &["U"]),
            scored(4, &["W", "R"]),
            scored(5, &["U"]),
            scored(6, &["G"]),
            scored(7, &["R", "W"]),
            scored(8, &["U"]),
            scored(9, &["U"]),
        ];
        let out = cap_color_identities(input, 2);
        assert_eq!(
            ids(&out),
            vec![1, 2, 4, 6, 7, 3, 5, 8, 9],
            "two of each identity stay, the capped blues follow in their old order"
        );
    }

    #[test]
    fn identity_order_does_not_matter() {
        let input = vec![
            scored(1, &["W", "R"]),
            scored(2, &["R", "W"]),
            scored(3, &["R"]),
        ];
        let out = cap_color_identities(input, 1);
        assert_eq!(ids(&out), vec![1, 3, 2]);
    }

    #[test]
    fn colorless_is_an_identity_too() {
        let input = vec![scored(1, &[]), scored(2, &[]), scored(3, &["B"])];
        let out = cap_color_identities(input, 1);
        assert_eq!(ids(&out), vec![1, 3, 2]);
    }

    fn deck(n: u128, colors: &[&str]) -> CommanderDeck {
        CommanderDeck {
            commander_oracle_id: Uuid::from_u128(n),
            commander_colors: colors.iter().map(|c| c.to_string()).collect(),
            mainboard_oracle_ids: vec![],
        }
    }

    fn affinity(color: &str, decks: u32) -> ColorAffinity {
        ColorAffinity {
            color: color.to_string(),
            decks,
        }
    }

    #[test]
    fn no_decks_is_no_profile() {
        assert_eq!(summarize_profile(&[], &[]), None);
    }

    #[test]
    fn colors_sort_by_decks_then_wubrg_and_omit_zeros() {
        let decks = vec![
            deck(1, &["R", "W"]),
            deck(2, &["W", "U", "R"]),
            deck(3, &["G"]),
            deck(4, &["R"]),
        ];
        let profile = summarize_profile(&decks, &[]).unwrap();
        assert_eq!(profile.deck_count, 4);
        assert_eq!(
            profile.colors,
            vec![
                affinity("R", 3),
                affinity("W", 2),
                affinity("U", 1),
                affinity("G", 1)
            ],
            "no black, blue before green on the tie"
        );
    }

    #[test]
    fn colorless_deck_contributes_no_color() {
        let decks = vec![deck(1, &[]), deck(2, &["B"])];
        let profile = summarize_profile(&decks, &[]).unwrap();
        assert_eq!(profile.colors, vec![affinity("B", 1)]);
    }

    #[test]
    fn top_identity_is_the_most_built_in_wubrg_order() {
        let decks = vec![
            deck(1, &["R", "W"]),
            deck(2, &["W", "R"]),
            deck(3, &["U"]),
            deck(4, &["U", "R", "W"]),
        ];
        let profile = summarize_profile(&decks, &[]).unwrap();
        assert_eq!(
            profile.top_identity,
            Some(ColorIdentityCount {
                colors: vec!["W".to_string(), "R".to_string()],
                decks: 2,
            })
        );
    }

    #[test]
    fn top_identity_tie_prefers_more_colors_then_wubrg() {
        let decks = vec![
            deck(1, &["U"]),
            deck(2, &["U"]),
            deck(3, &["R", "G"]),
            deck(4, &["G", "R"]),
            deck(5, &["W", "U"]),
            deck(6, &["U", "W"]),
        ];
        let profile = summarize_profile(&decks, &[]).unwrap();
        assert_eq!(
            profile.top_identity.map(|t| t.colors),
            Some(vec!["W".to_string(), "U".to_string()]),
            "two colors beat one, and WU comes before RG"
        );
    }

    #[test]
    fn top_identity_is_none_when_every_identity_is_unique() {
        let decks = vec![deck(1, &["U"]), deck(2, &["R"]), deck(3, &[])];
        let profile = summarize_profile(&decks, &[]).unwrap();
        assert_eq!(profile.top_identity, None);
    }

    #[test]
    fn a_single_deck_is_its_own_top_identity() {
        let decks = vec![deck(1, &["G", "B"])];
        let profile = summarize_profile(&decks, &[]).unwrap();
        assert_eq!(
            profile.top_identity,
            Some(ColorIdentityCount {
                colors: vec!["B".to_string(), "G".to_string()],
                decks: 1,
            })
        );
    }

    #[test]
    fn tags_sum_strength_across_decks_and_count_decks() {
        let decks = vec![deck(1, &["R"]), deck(2, &["R"]), deck(3, &["R"])];
        let profiles = vec![
            Some(serde_json::json!({ "treasure": 0.6, "sacrifice-outlet": 0.4 })),
            Some(serde_json::json!({ "sacrifice-outlet": 0.5, "lifegain": 0.5 })),
            None,
        ];
        let profile = summarize_profile(&decks, &profiles).unwrap();
        let got: Vec<(&str, u32)> = profile
            .tags
            .iter()
            .map(|t| (t.slug.as_str(), t.decks))
            .collect();
        assert_eq!(
            got,
            vec![("sacrifice-outlet", 2), ("treasure", 1), ("lifegain", 1)],
            "0.9 over two decks, then 0.6, then 0.5"
        );
        assert!(profile.tags.iter().all(|t| t.role.is_none()));
    }

    #[test]
    fn tags_are_capped_and_tie_on_slug() {
        let decks = vec![deck(1, &["R"])];
        let weights: serde_json::Map<String, serde_json::Value> = (0..12)
            .map(|i| (format!("tag-{i:02}"), serde_json::json!(0.1)))
            .collect();
        let profiles = vec![Some(serde_json::Value::Object(weights))];
        let profile = summarize_profile(&decks, &profiles).unwrap();
        let slugs: Vec<&str> = profile.tags.iter().map(|t| t.slug.as_str()).collect();
        assert_eq!(slugs.len(), PROFILE_TAG_SHOWN);
        assert_eq!(slugs.first(), Some(&"tag-00"));
        assert_eq!(slugs.last(), Some(&"tag-07"));
    }
}
