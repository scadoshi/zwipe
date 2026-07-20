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
/// Welcome dialog on first opening a deck profile.
pub const HINT_FIRST_DECK: &str = "first_deck";
/// Browsing dialog on the deck cards list (fires only once cards exist).
pub const HINT_DECK_CARDS: &str = "deck_cards";
/// Account management dialog on the profile screen.
pub const HINT_PROFILE: &str = "profile";
/// Explainer for the shared card-filter bottom sheet (add/remove/view screens).
pub const HINT_FILTER: &str = "filter";
/// Explainer for the import cards screen (source, add/replace, board).
pub const HINT_IMPORT: &str = "import";
/// Explainer for the export deck screen (board selection, copy decklist).
pub const HINT_EXPORT: &str = "export";
/// Explainer for the oracle-tag dictionary screen (letter browse, search).
pub const HINT_ORACLE_TAG_DICTIONARY: &str = "oracle_tag_dictionary";
/// Explainer for the oracle-tag example-cards browse (swipe navigation).
pub const HINT_OTAG_EXAMPLES: &str = "otag_examples";

/// Maximum length of a hint key.
pub const HINT_KEY_MAX_LEN: usize = 64;

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
            HINT_FIRST_LOGIN,
            HINT_ADD_DECK_CARDS,
            HINT_SWIPE_SELECT,
            HINT_REMOVE_DECK_CARDS,
            HINT_CREATE_DECK,
            HINT_EDIT_DECK,
            HINT_FIRST_DECK,
            HINT_DECK_CARDS,
            HINT_PROFILE,
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
}
