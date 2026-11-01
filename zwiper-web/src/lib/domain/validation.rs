//! The account form lines zwipe-ui does not carry: Register's password
//! pair and Forgot password's address check. The rest come from
//! [`zwipe_ui::account`].
//!
//! Each function returns the line to show under its field, or `None` when
//! the field is fine. The rules are zwipe-core's; these only pick the words.

use zwipe_core::domain::Email;
use zwipe_core::domain::auth::password;

/// The line under a password whose confirmation differs.
pub const PASSWORDS_DIFFER: &str = "Passwords do not match";

/// Forgot password's one line for any address that does not parse.
pub const INVALID_RESET_EMAIL: &str = "Please enter a valid email address";

/// Forgot password's check: the address as typed, untrimmed, with one fixed
/// line for every failure.
pub fn reset_email_error(raw: &str) -> Option<String> {
    Email::new(raw)
        .is_err()
        .then(|| INVALID_RESET_EMAIL.to_string())
}

/// A new password and its confirmation: the policy's first failure, else
/// [`PASSWORDS_DIFFER`]. A confirmation still shorter than the password is
/// not judged until a submit was attempted, since mid-typing it cannot match
/// yet.
pub fn password_error(password: &str, confirm: &str, attempted: bool) -> Option<String> {
    if let Err(e) = password::validate(password) {
        return Some(e.to_string());
    }
    let judged = attempted || confirm.chars().count() >= password.chars().count();
    (password != confirm && judged).then(|| PASSWORDS_DIFFER.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "Abcdef1!";

    #[test]
    fn the_reset_form_has_one_line_and_does_not_trim() {
        assert_eq!(reset_email_error("a@b.co"), None);
        assert_eq!(
            reset_email_error("nope").as_deref(),
            Some(INVALID_RESET_EMAIL)
        );
        assert_eq!(
            reset_email_error(" a@b.co").as_deref(),
            Some(INVALID_RESET_EMAIL)
        );
    }

    #[test]
    fn the_policy_wins_over_the_mismatch() {
        assert_eq!(
            password_error("short", "other", true).as_deref(),
            Some("must be at least 8 characters long")
        );
    }

    #[test]
    fn a_short_confirmation_waits_until_a_submit() {
        assert_eq!(password_error(GOOD, "Abc", false), None);
        assert_eq!(
            password_error(GOOD, "Abc", true).as_deref(),
            Some(PASSWORDS_DIFFER)
        );
        assert_eq!(
            password_error(GOOD, "Abcdef1?", false).as_deref(),
            Some(PASSWORDS_DIFFER),
            "as long as the password, so judged"
        );
        assert_eq!(password_error(GOOD, GOOD, false), None);
    }
}
