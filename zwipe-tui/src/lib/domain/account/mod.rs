//! The account screen's rules: each form validated the way the app does, and the copy the screen shows.

use chrono::{DateTime, Utc};
use zwipe_core::domain::auth::models::password;
use zwipe_core::domain::user::{Email, Username};

/// Seconds the server makes a user wait between verification emails.
pub(crate) const RESEND_COOLDOWN_SECS: u64 = 60;

/// Seconds the delete button stays disabled after the dialog opens, the app's `DELETE_COUNTDOWN_SECS`.
pub(crate) const DELETE_DELAY_SECS: u64 = 5;

/// What is wrong with a new password on Change password, in the app's order: the policy, then a mismatch once a submit was tried or the confirmation is as long, then sameness with the current one.
pub(crate) fn change_password_error(
    current: &str,
    new: &str,
    confirm: &str,
    attempted: bool,
) -> Option<String> {
    if let Err(e) = password::validate(new) {
        return Some(e.to_string());
    }
    let long_enough = confirm.chars().count() >= new.chars().count();
    if new != confirm && (attempted || long_enough) {
        return Some("Passwords do not match".to_owned());
    }
    (new == current).then(|| "New password must be different from your current password".to_owned())
}

/// Why `raw` is not a username, in the app's words, or `None` when it is one.
pub(crate) fn username_error(raw: &str) -> Option<String> {
    Username::new(raw).err().map(|e| e.to_string())
}

/// Why `raw` is not an email address, in the app's words, or `None` when it is one.
pub(crate) fn email_error(raw: &str) -> Option<String> {
    Email::new(raw).err().map(|e| email_message(&e).to_owned())
}

/// The app's wording for each way an email address can be wrong.
fn email_message(error: &email_address::Error) -> &'static str {
    use email_address::Error::{
        DomainEmpty, DomainInvalidSeparator, DomainTooFew, DomainTooLong, InvalidCharacter,
        InvalidComment, InvalidIPAddress, LocalPartEmpty, LocalPartTooLong, MissingDisplayName,
        MissingEndBracket, MissingSeparator, SubDomainEmpty, SubDomainTooLong, UnbalancedQuotes,
        UnsupportedDisplayName, UnsupportedDomainLiteral,
    };
    match error {
        InvalidCharacter => "invalid character in email",
        MissingSeparator => "missing @ symbol",
        LocalPartEmpty => "missing text before @",
        LocalPartTooLong => "text before @ is too long",
        DomainEmpty => "missing domain after @",
        DomainTooLong => "domain is too long",
        SubDomainEmpty => "empty part in domain",
        SubDomainTooLong => "a part of the domain is too long",
        DomainTooFew => "domain must have at least one dot",
        DomainInvalidSeparator => "invalid dot placement in domain",
        UnbalancedQuotes => "unbalanced quotes in email",
        InvalidComment => "invalid comment in email",
        InvalidIPAddress => "invalid ip address in domain",
        UnsupportedDomainLiteral => "domain literal not supported",
        UnsupportedDisplayName => "display name not supported",
        MissingDisplayName => "missing display name",
        MissingEndBracket => "missing closing bracket",
    }
}

/// "Password is required" for an empty password.
pub(crate) fn password_required(password: &str) -> Option<String> {
    password
        .is_empty()
        .then(|| "Password is required".to_owned())
}

/// The note under a pending email change, the app's `link_expiry_text`: how long its link lasts, or that it has expired. Minutes round up; from an hour on, minutes round to the nearest hour.
pub(crate) fn pending_note(expires_at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let seconds = expires_at.signed_duration_since(now).num_seconds();
    if seconds <= 0 {
        return "Link expired; tap Change to send a new one".to_owned();
    }
    let minutes = seconds.saturating_add(59) / 60;
    if minutes < 60 {
        return plural("Link expires in", minutes, "minute");
    }
    let hours = minutes.saturating_add(30) / 60;
    plural("Link expires in", hours, "hour")
}

fn plural(prefix: &str, n: i64, unit: &str) -> String {
    if n == 1 {
        format!("{prefix} 1 {unit}")
    } else {
        format!("{prefix} {n} {unit}s")
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    #[test]
    fn usernames_are_checked_by_zwipe_core() {
        assert_eq!(username_error("scotty"), None);
        assert_eq!(
            username_error("ab").as_deref(),
            Some("must be at least 3 characters long")
        );
        assert_eq!(
            username_error("has space").as_deref(),
            Some("cannot contain whitespace")
        );
    }

    #[test]
    fn emails_use_the_apps_wording() {
        assert_eq!(email_error("me@zwipe.net"), None);
        assert_eq!(
            email_error("me.zwipe.net").as_deref(),
            Some("missing @ symbol")
        );
        assert_eq!(
            email_error("me@localhost").as_deref(),
            Some("domain must have at least one dot")
        );
    }

    #[test]
    fn change_password_waits_on_the_mismatch_until_the_confirmation_is_long_enough() {
        assert_eq!(
            password_required("").as_deref(),
            Some("Password is required")
        );
        assert_eq!(password_required("x"), None);
        let check = |confirm, attempted| {
            change_password_error("Old!pass1", "New!pass22", confirm, attempted)
        };
        assert_eq!(check("New", false), None, "still typing");
        assert_eq!(
            check("New", true).as_deref(),
            Some("Passwords do not match")
        );
        assert_eq!(
            check("New!pass23", false).as_deref(),
            Some("Passwords do not match")
        );
        assert_eq!(check("New!pass22", false), None);
        assert_eq!(
            change_password_error("Same!pass1", "Same!pass1", "Same!pass1", true).as_deref(),
            Some("New password must be different from your current password")
        );
    }

    #[test]
    fn the_pending_note_counts_down_in_minutes_then_hours() {
        let now = Utc::now();
        let note = |d: Duration| pending_note(now + d, now);
        assert_eq!(note(Duration::seconds(30)), "Link expires in 1 minute");
        assert_eq!(note(Duration::minutes(45)), "Link expires in 45 minutes");
        assert_eq!(note(Duration::hours(1)), "Link expires in 1 hour");
        assert_eq!(note(Duration::hours(23)), "Link expires in 23 hours");
        assert_eq!(note(Duration::seconds(3599)), "Link expires in 1 hour");
        assert_eq!(
            note(Duration::minutes(89) + Duration::seconds(30)),
            "Link expires in 2 hours"
        );
        assert_eq!(note(Duration::minutes(59)), "Link expires in 59 minutes");
        assert_eq!(
            note(Duration::seconds(-5)),
            "Link expired; tap Change to send a new one"
        );
    }
}
