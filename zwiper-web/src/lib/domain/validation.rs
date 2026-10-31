//! The live error lines of the account forms: Register, and Account's change
//! username, email and password sheets.
//!
//! Each function returns the line to show under its field, or `None` when
//! the field is fine. The rules are zwipe-core's; these only pick the words.

use zwipe_core::domain::Email;
use zwipe_core::domain::auth::password;
use zwipe_core::domain::user::username::Username;

/// The line under a password whose confirmation differs.
pub const PASSWORDS_DIFFER: &str = "Passwords do not match";

/// The line under an empty password on a sheet that needs one.
pub const PASSWORD_REQUIRED: &str = "Password is required";

/// The line under a new password equal to the current one.
pub const PASSWORD_UNCHANGED: &str = "New password must be different from your current password";

/// Forgot password's one line for any address that does not parse.
pub const INVALID_RESET_EMAIL: &str = "Please enter a valid email address";

/// The username rules' first failure, as zwipe-core words it. The username
/// is trimmed before it is judged.
pub fn username_error(raw: &str) -> Option<String> {
    Username::new(raw).err().map(|e| e.to_string())
}

/// The email rules' failure for `raw` as given, in plain words.
pub fn email_error(raw: &str) -> Option<String> {
    Email::new(raw).err().map(|e| email_words(&e))
}

/// Forgot password's check: the address as typed, untrimmed, with one fixed
/// line for every failure.
pub fn reset_email_error(raw: &str) -> Option<String> {
    Email::new(raw)
        .is_err()
        .then(|| INVALID_RESET_EMAIL.to_string())
}

/// A sheet's password field, which only has to be filled.
pub fn required_password_error(password: &str) -> Option<String> {
    password.is_empty().then(|| PASSWORD_REQUIRED.to_string())
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

/// Change password's line under the new password: [`password_error`], then
/// [`PASSWORD_UNCHANGED`] when it equals `current`. The current password
/// itself is never judged, so a user whose password predates the policy can
/// still change it.
pub fn new_password_error(
    current: &str,
    password: &str,
    confirm: &str,
    attempted: bool,
) -> Option<String> {
    password_error(password, confirm, attempted)
        .or_else(|| (password == current).then(|| PASSWORD_UNCHANGED.to_string()))
}

/// The plain words for each email parser failure.
fn email_words(e: &email_address::Error) -> String {
    use email_address::Error;
    match e {
        Error::InvalidCharacter => "invalid character in email",
        Error::MissingSeparator => "missing @ symbol",
        Error::LocalPartEmpty => "missing text before @",
        Error::LocalPartTooLong => "text before @ is too long",
        Error::DomainEmpty => "missing domain after @",
        Error::DomainTooLong => "domain is too long",
        Error::SubDomainEmpty => "empty part in domain",
        Error::SubDomainTooLong => "a part of the domain is too long",
        Error::DomainTooFew => "domain must have at least one dot",
        Error::DomainInvalidSeparator => "invalid dot placement in domain",
        Error::UnbalancedQuotes => "unbalanced quotes in email",
        Error::InvalidComment => "invalid comment in email",
        Error::InvalidIPAddress => "invalid ip address in domain",
        Error::UnsupportedDomainLiteral => "domain literal not supported",
        Error::UnsupportedDisplayName => "display name not supported",
        Error::MissingDisplayName => "missing display name",
        Error::MissingEndBracket => "missing closing bracket",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "Abcdef1!";

    #[test]
    fn usernames_take_core_rules_in_core_words() {
        assert_eq!(username_error("alice"), None);
        assert_eq!(username_error("  alice  "), None, "trimmed first");
        assert_eq!(
            username_error("al").as_deref(),
            Some("must be at least 3 characters long")
        );
        assert_eq!(
            username_error("al ice").as_deref(),
            Some("cannot contain whitespace")
        );
    }

    #[test]
    fn emails_name_what_is_wrong() {
        assert_eq!(email_error("a@b.co"), None);
        assert_eq!(email_error("ab.co").as_deref(), Some("missing @ symbol"));
        assert_eq!(
            email_error("@b.co").as_deref(),
            Some("missing text before @")
        );
        assert_eq!(
            email_error("a@b").as_deref(),
            Some("domain must have at least one dot")
        );
    }

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

    #[test]
    fn a_new_password_must_differ_from_the_current_one() {
        assert_eq!(
            new_password_error(GOOD, GOOD, GOOD, true).as_deref(),
            Some(PASSWORD_UNCHANGED)
        );
        assert_eq!(new_password_error("old", GOOD, GOOD, true), None);
        assert_eq!(
            new_password_error(GOOD, GOOD, "x", true).as_deref(),
            Some(PASSWORDS_DIFFER),
            "the mismatch comes first"
        );
    }

    #[test]
    fn a_sheet_password_only_has_to_be_filled() {
        assert_eq!(
            required_password_error("").as_deref(),
            Some(PASSWORD_REQUIRED)
        );
        assert_eq!(required_password_error(" "), None);
    }
}
