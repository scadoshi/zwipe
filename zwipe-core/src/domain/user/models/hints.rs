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
/// Swipe vocabulary dialog on the add-cards screen.
pub const HINT_ADD_SWIPES: &str = "add_swipes";
/// Swipe vocabulary dialog on the remove-cards screen.
pub const HINT_REMOVE_SWIPES: &str = "remove_swipes";
/// Welcome dialog on first opening a deck profile.
pub const HINT_FIRST_DECK: &str = "first_deck";
/// Browsing dialog on the deck cards list (fires only once cards exist).
pub const HINT_DECK_CARDS: &str = "deck_cards";
/// Account management dialog on the profile screen.
pub const HINT_PROFILE: &str = "profile";

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
            HINT_ADD_SWIPES,
            HINT_REMOVE_SWIPES,
            HINT_FIRST_DECK,
            HINT_DECK_CARDS,
            HINT_PROFILE,
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
        assert_eq!(MarkHintShown::new(id, "").unwrap_err(), InvalidHintKey::Empty);
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
