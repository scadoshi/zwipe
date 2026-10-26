//! Suggested commanders: the user's commander decks scored against the
//! precomputed commander tag profiles, the variety cap over the ranking, and
//! the dismissals that hide one.

use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;
use zwipe_core::domain::card::Card;

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

/// The ranked suggestions and the deck count that fed them.
#[derive(Debug, Clone)]
pub struct CommanderSuggestions {
    /// Ranked best first.
    pub suggestions: Vec<CommanderSuggestion>,
    /// The user's decks with a commander. 0 means no profile could be built.
    pub deck_count: u32,
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
}
