//! Suggested commanders: the user's commander decks scored against the
//! precomputed commander tag profiles, and the dismissals that hide one.

use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;
use zwipe_core::domain::card::Card;

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
pub struct SharedTheme {
    /// The oracle tag slug.
    pub slug: String,
    /// The tag's label from the catalog; the slug when it has none.
    pub label: String,
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
#[derive(Debug, Clone)]
pub struct ScoredCommander {
    /// The commander's oracle id.
    pub oracle_id: Uuid,
    /// The strongest shared tags, strongest first.
    pub themes: Vec<SharedTheme>,
}

/// One suggestion with the terms behind it.
#[derive(Debug, Clone)]
pub struct CommanderSuggestion {
    /// The commander's current printing.
    pub card: Card,
    /// Shared themes, strongest first.
    pub themes: Vec<SharedTheme>,
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
