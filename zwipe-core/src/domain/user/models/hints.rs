//! One-time UI hint tracking.
//!
//! Hints are lightweight, contextual teaching moments (a dialog explaining
//! swipe directions, a first-deck welcome) shown at most once per account.
//! Shown hints live in a `hints_shown` map on the user, keyed by the
//! identifiers below; a new hint is just a new key, no migration needed.
//! Clients ignore keys they don't know.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Welcome dialog on the home screen after first login.
pub const HINT_FIRST_LOGIN: &str = "first_login";
/// Swipe vocabulary dialog on the add deck cards screen.
pub const HINT_ADD_DECK_CARDS: &str = "add_deck_cards";
/// Swipe vocabulary dialog on the Swipe select (commander) screen.
pub const HINT_SWIPE_SELECT: &str = "swipe_select";
/// Swipe vocabulary dialog on the remove deck cards screen.
pub const HINT_REMOVE_DECK_CARDS: &str = "remove_deck_cards";
/// Explainer for the create deck form (name, format, command zone).
pub const HINT_CREATE_DECK: &str = "create_deck";
/// Explainer for the edit deck form (name, format, command zone).
pub const HINT_EDIT_DECK: &str = "edit_deck";
/// Browsing dialog on the deck list (fires only once decks exist; its main
/// job is teaching the group/filter chip rows, which an empty list hides).
pub const HINT_DECK_LIST: &str = "deck_list";
/// Welcome dialog on first opening a deck profile.
pub const HINT_FIRST_DECK: &str = "first_deck";
/// Browsing dialog on the deck cards list (fires only once cards exist).
pub const HINT_DECK_CARDS: &str = "deck_cards";
/// Account management dialog on the profile screen.
pub const HINT_PROFILE: &str = "profile";
/// Explainer for the Config screen (appearance and card settings).
pub const HINT_CONFIG: &str = "config";
/// Explainer for the shared card-filter bottom sheet (add/remove/view screens).
pub const HINT_FILTER: &str = "filter";
/// Explainer for the Universes Beyond exceptions sheet on the Config screen.
pub const HINT_UNIVERSES_BEYOND_EXCEPTIONS: &str = "universes_beyond_exceptions";
/// Explainer for the import cards screen (source, add/replace, board).
pub const HINT_IMPORT: &str = "import";
/// Explainer for the export deck screen (board selection, copy decklist).
pub const HINT_EXPORT: &str = "export";
/// Explainer for the oracle-tag dictionary screen (letter browse, search).
pub const HINT_ORACLE_TAG_DICTIONARY: &str = "oracle_tag_dictionary";
/// Explainer for the oracle-tag example-cards browse (swipe navigation).
pub const HINT_OTAG_EXAMPLES: &str = "otag_examples";
/// Explainer for the commander maybeboard screen (up-swipe origin, actions).
pub const HINT_COMMANDER_MAYBEBOARD: &str = "commander_maybeboard";
/// Explainer for the Playstyle screen (the sections, row actions,
/// Swipe, top 10). The key keeps its suggested commanders name.
pub const HINT_SUGGESTED_COMMANDERS: &str = "suggested_commanders";

/// Explainer for the Activity screen (the heatmap, swipes, rates and hard
/// passes).
pub const HINT_ACTIVITY: &str = "activity";

/// Maximum length of a hint key.
pub const HINT_KEY_MAX_LEN: usize = 64;

/// Every one-time hint a client shows, each recorded under one of the
/// `HINT_*` keys above.
///
/// Clients name a hint by this enum and send [`OneTimeHint::key`] to the
/// server, so a typo in a key is a compile error instead of a hint that
/// shows again on every visit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OneTimeHint {
    /// Home, on the first visit after the first login.
    FirstLogin,
    /// Account, on mount.
    Profile,
    /// Config, on mount.
    Config,
    /// The first time the Universes Beyond exceptions open on Config.
    UniversesBeyondExceptions,
    /// The deck list, once it has a deck.
    DeckList,
    /// Create deck, on mount.
    CreateDeck,
    /// Edit deck, on mount.
    EditDeck,
    /// The deck view, on mount.
    FirstDeck,
    /// The deck cards view, once the deck has cards.
    DeckCards,
    /// Add cards, on mount.
    AddDeckCards,
    /// Remove cards, on mount.
    RemoveDeckCards,
    /// The first use of the card filter.
    Filter,
    /// Import, on mount.
    Import,
    /// Export, on mount.
    Export,
    /// The oracle tag dictionary, on mount.
    OracleTagDictionary,
    /// Oracle tag examples, on mount.
    OtagExamples,
    /// The commander maybeboard, on mount.
    CommanderMaybeboard,
    /// Playstyle, on mount. The key keeps its suggested commanders name.
    SuggestedCommanders,
    /// Activity, on mount.
    Activity,
    /// The first time Swipe select opens.
    SwipeSelect,
}

impl OneTimeHint {
    /// Every hint, in the order the `HINT_*` keys are declared.
    pub const ALL: [Self; 20] = [
        Self::FirstLogin,
        Self::AddDeckCards,
        Self::SwipeSelect,
        Self::RemoveDeckCards,
        Self::CreateDeck,
        Self::EditDeck,
        Self::DeckList,
        Self::FirstDeck,
        Self::DeckCards,
        Self::Profile,
        Self::Config,
        Self::Filter,
        Self::UniversesBeyondExceptions,
        Self::Import,
        Self::Export,
        Self::OracleTagDictionary,
        Self::OtagExamples,
        Self::CommanderMaybeboard,
        Self::SuggestedCommanders,
        Self::Activity,
    ];

    /// The key the server records the hint under.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::FirstLogin => HINT_FIRST_LOGIN,
            Self::Profile => HINT_PROFILE,
            Self::Config => HINT_CONFIG,
            Self::UniversesBeyondExceptions => HINT_UNIVERSES_BEYOND_EXCEPTIONS,
            Self::DeckList => HINT_DECK_LIST,
            Self::CreateDeck => HINT_CREATE_DECK,
            Self::EditDeck => HINT_EDIT_DECK,
            Self::FirstDeck => HINT_FIRST_DECK,
            Self::DeckCards => HINT_DECK_CARDS,
            Self::AddDeckCards => HINT_ADD_DECK_CARDS,
            Self::RemoveDeckCards => HINT_REMOVE_DECK_CARDS,
            Self::Filter => HINT_FILTER,
            Self::Import => HINT_IMPORT,
            Self::Export => HINT_EXPORT,
            Self::OracleTagDictionary => HINT_ORACLE_TAG_DICTIONARY,
            Self::OtagExamples => HINT_OTAG_EXAMPLES,
            Self::CommanderMaybeboard => HINT_COMMANDER_MAYBEBOARD,
            Self::SuggestedCommanders => HINT_SUGGESTED_COMMANDERS,
            Self::Activity => HINT_ACTIVITY,
            Self::SwipeSelect => HINT_SWIPE_SELECT,
        }
    }

    /// The hint recorded under `key`, or `None` for a key this build does not
    /// know, which clients ignore.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|hint| hint.key() == key)
    }
}

/// Validated request to mark a one-time hint as shown for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkHintShown {
    /// User the hint was shown to.
    pub user_id: Uuid,
    /// Validated hint key (lowercase snake case, max 64 chars).
    pub hint: String,
}

impl MarkHintShown {
    /// Validates and constructs the request.
    ///
    /// Keys are deliberately not checked against a fixed list: new hints are
    /// shipped client-side as new keys without a server deploy. Validation
    /// only enforces the key shape.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidHintKey`] when the trimmed key is empty, longer than
    /// [`HINT_KEY_MAX_LEN`] bytes, or has characters outside lowercase snake
    /// case.
    pub fn new(user_id: Uuid, hint: &str) -> Result<Self, InvalidHintKey> {
        let hint = hint.trim();
        if hint.is_empty() {
            return Err(InvalidHintKey::Empty);
        }
        if hint.len() > HINT_KEY_MAX_LEN {
            return Err(InvalidHintKey::TooLong);
        }
        if !hint
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(InvalidHintKey::InvalidCharacters);
        }
        Ok(Self {
            user_id,
            hint: hint.to_string(),
        })
    }
}

/// Validation error for hint keys.
#[derive(Debug, Error, PartialEq)]
pub enum InvalidHintKey {
    /// Key is empty or whitespace.
    #[error("hint key is empty")]
    Empty,
    /// Key exceeds the maximum length.
    #[error("hint key is too long")]
    TooLong,
    /// Key contains characters outside lowercase snake case.
    #[error("hint key must be lowercase snake case")]
    InvalidCharacters,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_hint_keys() {
        for key in [
            HINT_ACTIVITY,
            HINT_FIRST_LOGIN,
            HINT_ADD_DECK_CARDS,
            HINT_SWIPE_SELECT,
            HINT_REMOVE_DECK_CARDS,
            HINT_CREATE_DECK,
            HINT_EDIT_DECK,
            HINT_DECK_LIST,
            HINT_FIRST_DECK,
            HINT_DECK_CARDS,
            HINT_PROFILE,
            HINT_CONFIG,
            HINT_FILTER,
            HINT_IMPORT,
            HINT_EXPORT,
            HINT_ORACLE_TAG_DICTIONARY,
        ] {
            assert!(MarkHintShown::new(Uuid::new_v4(), key).is_ok());
        }
    }

    #[test]
    fn trims_and_accepts_snake_case() {
        let request = MarkHintShown::new(Uuid::new_v4(), "  some_hint_2  ");
        assert_eq!(request.map(|r| r.hint).ok().as_deref(), Some("some_hint_2"));
    }

    #[test]
    fn rejects_bad_keys() {
        let id = Uuid::new_v4();
        assert_eq!(
            MarkHintShown::new(id, "").unwrap_err(),
            InvalidHintKey::Empty
        );
        assert_eq!(
            MarkHintShown::new(id, "   ").unwrap_err(),
            InvalidHintKey::Empty
        );
        assert_eq!(
            MarkHintShown::new(id, &"a".repeat(65)).unwrap_err(),
            InvalidHintKey::TooLong
        );
        assert_eq!(
            MarkHintShown::new(id, "Add Swipes").unwrap_err(),
            InvalidHintKey::InvalidCharacters
        );
        assert_eq!(
            MarkHintShown::new(id, "add-swipes").unwrap_err(),
            InvalidHintKey::InvalidCharacters
        );
    }

    #[test]
    fn every_one_time_hint_has_its_own_valid_key() {
        let mut keys: Vec<&str> = OneTimeHint::ALL.iter().map(|h| h.key()).collect();
        for key in &keys {
            assert!(MarkHintShown::new(Uuid::new_v4(), key).is_ok(), "{key}");
        }
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), OneTimeHint::ALL.len(), "keys are distinct");
    }

    #[test]
    fn one_time_hints_round_trip_through_their_keys() {
        for hint in OneTimeHint::ALL {
            assert_eq!(OneTimeHint::from_key(hint.key()), Some(hint));
        }
        assert_eq!(OneTimeHint::from_key("not_a_hint"), None);
        assert_eq!(
            OneTimeHint::from_key(HINT_UNIVERSES_BEYOND_EXCEPTIONS),
            Some(OneTimeHint::UniversesBeyondExceptions)
        );
    }
}
