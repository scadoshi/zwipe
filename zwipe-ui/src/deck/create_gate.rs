//! The check every Create deck action runs before it opens the create screen.

use zwipe_core::domain::deck::UNVERIFIED_MAX_DECKS_PER_USER;

use crate::toast::{LONG, Toast, ToastKind};

/// The warning an unverified account sees once it holds as many decks as it may.
pub const VERIFY_TO_CREATE: &str = "Verify your email to create more than 1 deck";

/// Whether the user may start a new deck, given whether their email is verified and how many decks they hold now.
///
/// A verified account always passes: its cap is the server's to report. The count is the client's loaded deck list, so a list that hasn't loaded counts as 0 and passes, and the server still refuses an over-cap create.
///
/// # Errors
///
/// An unverified account holding [`UNVERIFIED_MAX_DECKS_PER_USER`] decks or more gets the [`VERIFY_TO_CREATE`] warning, up for [`LONG`].
pub fn create_gate(verified: bool, deck_count: usize) -> Result<(), Toast> {
    let at_limit = i64::try_from(deck_count).map_or(true, |n| n >= UNVERIFIED_MAX_DECKS_PER_USER);
    if verified || !at_limit {
        return Ok(());
    }
    Err(Toast::new(ToastKind::Warning, VERIFY_TO_CREATE, LONG))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_verified_account_always_passes() {
        for count in [0, 1, 2, 20, usize::MAX] {
            assert_eq!(create_gate(true, count), Ok(()), "count {count}");
        }
    }

    #[test]
    fn an_unverified_account_with_no_deck_passes() {
        assert_eq!(create_gate(false, 0), Ok(()));
    }

    #[test]
    fn an_unverified_account_at_or_over_the_cap_is_warned() {
        let warning = Toast::new(ToastKind::Warning, VERIFY_TO_CREATE, LONG);
        for count in [1, 2, 20, usize::MAX] {
            assert_eq!(
                create_gate(false, count),
                Err(warning.clone()),
                "count {count}"
            );
        }
    }

    #[test]
    fn the_warning_matches_the_app() {
        let Err(toast) = create_gate(false, 1) else {
            panic!("an unverified account with a deck is gated");
        };
        assert_eq!(toast.kind, ToastKind::Warning);
        assert_eq!(toast.text, "Verify your email to create more than 1 deck");
        assert_eq!(toast.duration, LONG);
    }

    #[test]
    fn the_warning_names_the_cap() {
        let cap = format!("more than {UNVERIFIED_MAX_DECKS_PER_USER} deck");
        assert!(
            VERIFY_TO_CREATE.contains(&cap),
            "{VERIFY_TO_CREATE:?} vs {cap:?}"
        );
    }
}
