//! The account forms' rules, as the app has them: Log in, Register, Forgot password and the Account screen. Each check returns the line to show under its field, or `None` when the field is fine; the policy is zwipe-core's and only the words are picked here.

use std::fmt::{self, Debug, Formatter};

use chrono::{DateTime, Utc};
use zwipe_core::domain::user::{Email, InvalidUsername, Username};
use zwipe_core::text::sentence_case;

use crate::toast::{Toast, ToastKind};

/// The toast for a Log in tap with a field left empty.
pub const MISSING_INPUTS: &str = "Enter your username or email and password";

/// The one line under any address that does not parse, on every form.
pub const INVALID_EMAIL: &str = "Enter a valid email address";

/// The line under an empty email field, on every form.
pub const EMAIL_REQUIRED: &str = "Email is required";

/// The line under a username the word filter turns down.
pub const USERNAME_BAD_WORD: &str = "Pick a different username";

/// The line under a password whose confirmation differs.
pub const PASSWORDS_DIFFER: &str = "Passwords do not match";

/// The line under an empty password on Change username and Change email.
pub const PASSWORD_REQUIRED: &str = "Password is required";

/// The line under a new password that repeats the current one.
pub const PASSWORD_UNCHANGED: &str = "New password must be different from your current password";

/// The line under a new username that repeats the current one.
pub const USERNAME_UNCHANGED: &str = "New username must be different from your current username";

/// The line under a new email address that repeats the current one.
pub const EMAIL_UNCHANGED: &str = "New email must be different from your current email";

/// A password as typed, carried by an action, a request or a screen's state. Its `Debug` hides it; [`read`](Self::read) is the way to the text.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Password(String);

impl Password {
    /// Wraps a password as typed.
    #[must_use]
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// The password as typed, for a check or the request body.
    #[must_use]
    pub fn read(&self) -> &str {
        &self.0
    }

    /// Whether nothing is typed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// By hand: the derive would print the password.
impl Debug for Password {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Password(REDACTED)")
    }
}

/// What a tap on Log in does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoginGate {
    /// A request is already out: ignore the tap.
    Busy,
    /// Nothing to send: raise [`missing_inputs_toast`].
    MissingInputs,
    /// Send the request.
    Send,
}

/// Decides a Log in tap from the loading flag and the two fields.
///
/// The identifier counts as empty when it is only whitespace, but it is sent as typed. The password is checked for presence only: login applies no policy, since a rule that tightened later would lock out an account whose credentials predate it, and the server is the only judge of a credential.
#[must_use]
pub fn login_gate(is_loading: bool, identifier: &str, password: &str) -> LoginGate {
    if is_loading {
        LoginGate::Busy
    } else if identifier.trim().is_empty() || password.is_empty() {
        LoginGate::MissingInputs
    } else {
        LoginGate::Send
    }
}

/// [`MISSING_INPUTS`] as the app raises it: an error, up for [`NORMAL`](crate::toast::NORMAL).
#[must_use]
pub fn missing_inputs_toast() -> Toast {
    Toast::normal(ToastKind::Error, MISSING_INPUTS)
}

/// Forgot password's check: [`EMAIL_REQUIRED`] when nothing but spaces is typed, else none. The address belongs to an existing account, so its form is the server's to judge: one accepted under older rules must still get a link.
#[must_use]
pub fn reset_email_error(raw: &str) -> Option<String> {
    raw.trim().is_empty().then(|| EMAIL_REQUIRED.to_owned())
}

/// Register's check of a new password and its confirmation: [`PASSWORD_REQUIRED`] once a submit was tried with both empty, else [`PASSWORDS_DIFFER`]. The password policy is the server's alone, so no rule a later policy changes can stop a client from sending. A confirmation still shorter than the password is not judged until a submit was tried, since mid-typing it cannot match yet.
#[must_use]
pub fn password_error(password: &str, confirm: &str, attempted: bool) -> Option<String> {
    if password.is_empty() && confirm.is_empty() {
        return attempted.then(|| PASSWORD_REQUIRED.to_owned());
    }
    let judged = attempted || confirm.chars().count() >= password.chars().count();
    (password != confirm && judged).then(|| PASSWORDS_DIFFER.to_owned())
}

/// What is wrong with a new password on Change password, in the app's order: Register's [`password_error`], then [`PASSWORD_UNCHANGED`] when it repeats the current one.
#[must_use]
pub fn change_password_error(
    current: &str,
    new: &str,
    confirm: &str,
    attempted: bool,
) -> Option<String> {
    password_error(new, confirm, attempted)
        .or_else(|| (new == current).then(|| PASSWORD_UNCHANGED.to_owned()))
}

/// Why `raw` is not a username, in the app's words, or `None` when it is one.
#[must_use]
pub fn username_error(raw: &str) -> Option<String> {
    Username::new(raw).err().map(|e| match e {
        InvalidUsername::BadWord => USERNAME_BAD_WORD.to_owned(),
        InvalidUsername::Whitespace => "Can't contain spaces".to_owned(),
        other => sentence_case(&other.to_string()),
    })
}

/// [`EMAIL_REQUIRED`] when `raw` is empty or only spaces, [`INVALID_EMAIL`] when it is not an email address, `None` when it is one.
#[must_use]
pub fn email_error(raw: &str) -> Option<String> {
    if raw.trim().is_empty() {
        return Some(EMAIL_REQUIRED.to_owned());
    }
    Email::new(raw).is_err().then(|| INVALID_EMAIL.to_owned())
}

/// What is wrong with a new username on Change username: [`username_error`], then [`USERNAME_UNCHANGED`] when it is exactly the current one. Usernames are unique by exact text, so a change of case is a change.
#[must_use]
pub fn change_username_error(current: &str, raw: &str) -> Option<String> {
    username_error(raw).or_else(|| (raw == current).then(|| USERNAME_UNCHANGED.to_owned()))
}

/// What is wrong with a new address on Change email: [`email_error`], then [`EMAIL_UNCHANGED`] when it is exactly the current one.
#[must_use]
pub fn change_email_error(current: &str, raw: &str) -> Option<String> {
    email_error(raw).or_else(|| (raw == current).then(|| EMAIL_UNCHANGED.to_owned()))
}

/// [`PASSWORD_REQUIRED`] for an empty password.
#[must_use]
pub fn password_required(password: &str) -> Option<String> {
    password.is_empty().then(|| PASSWORD_REQUIRED.to_owned())
}

/// The Resend button's label on the email verification row: "Sending…" while a send is out, "Resend in {n}s" while `cooldown_secs` runs, otherwise "Resend".
#[must_use]
pub fn resend_label(sending: bool, cooldown_secs: u64) -> String {
    if sending {
        "Sending…".to_owned()
    } else if cooldown_secs > 0 {
        format!("Resend in {cooldown_secs}s")
    } else {
        "Resend".to_owned()
    }
}

/// The Delete account button's label with `remaining_secs` left on its countdown: "Delete ({n})" while it counts, "Delete" once it can be pressed.
#[must_use]
pub fn delete_label(remaining_secs: u64) -> String {
    match remaining_secs {
        0 => "Delete".to_owned(),
        n => format!("Delete ({n})"),
    }
}

/// The note under a pending email change, the app's `link_expiry_text`: how long its link lasts, or that it has expired. The link is expired only once no time at all is left. Under a minute left reads "less than a minute"; otherwise seconds round up to minutes, and from an hour on, minutes round to the nearest hour.
#[must_use]
pub fn pending_note(expires_at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let left = expires_at.signed_duration_since(now);
    if left <= chrono::Duration::zero() {
        return "Link expired; tap Change to send a new one".to_owned();
    }
    if left < chrono::Duration::minutes(1) {
        return "Link expires in less than a minute".to_owned();
    }
    let minutes = left.num_seconds().saturating_add(59) / 60;
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
            Some("Must be at least 3 characters long")
        );
        assert_eq!(
            username_error("has space").as_deref(),
            Some("Can't contain spaces")
        );
    }

    #[test]
    fn every_bad_email_gets_the_same_plain_line() {
        assert_eq!(email_error("me@zwipe.net"), None);
        for raw in [
            "me.zwipe.net",
            "me@localhost",
            "\"me@zwipe.net",
            "me@[127.0.0.1]",
        ] {
            assert_eq!(email_error(raw).as_deref(), Some(INVALID_EMAIL), "{raw:?}");
        }
    }

    #[test]
    fn an_empty_email_is_required_not_invalid() {
        for raw in ["", "   "] {
            assert_eq!(email_error(raw).as_deref(), Some(EMAIL_REQUIRED), "{raw:?}");
            assert_eq!(
                reset_email_error(raw).as_deref(),
                Some(EMAIL_REQUIRED),
                "{raw:?}"
            );
        }
        assert_eq!(EMAIL_REQUIRED, "Email is required");
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
        assert_eq!(
            note(Duration::seconds(30)),
            "Link expires in less than a minute"
        );
        assert_eq!(note(Duration::seconds(60)), "Link expires in 1 minute");
        assert_eq!(note(Duration::seconds(61)), "Link expires in 2 minutes");
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
        assert_eq!(
            note(Duration::zero()),
            "Link expired; tap Change to send a new one"
        );
    }

    #[test]
    fn under_a_second_left_is_not_yet_expired() {
        let now = Utc::now();
        assert_eq!(
            pending_note(now + Duration::milliseconds(500), now),
            "Link expires in less than a minute"
        );
    }

    const GOOD: &str = "Abcdef1!";

    #[test]
    fn the_shared_lines_keep_the_apps_words() {
        assert_eq!(MISSING_INPUTS, "Enter your username or email and password");
        assert_eq!(INVALID_EMAIL, "Enter a valid email address");
        assert_eq!(PASSWORDS_DIFFER, "Passwords do not match");
        assert_eq!(PASSWORD_REQUIRED, "Password is required");
        assert_eq!(
            PASSWORD_UNCHANGED,
            "New password must be different from your current password"
        );
        assert_eq!(
            USERNAME_UNCHANGED,
            "New username must be different from your current username"
        );
        assert_eq!(
            EMAIL_UNCHANGED,
            "New email must be different from your current email"
        );
    }

    #[test]
    fn an_unchanged_username_or_email_is_refused_and_a_case_change_is_not() {
        assert_eq!(
            change_username_error("scadoshi", "scadoshi"),
            Some(USERNAME_UNCHANGED.to_owned())
        );
        assert_eq!(change_username_error("scadoshi", "Scadoshi"), None);
        assert_eq!(change_username_error("scadoshi", "someone"), None);
        assert_eq!(
            change_email_error("a@example.com", "a@example.com"),
            Some(EMAIL_UNCHANGED.to_owned())
        );
        assert_eq!(change_email_error("a@example.com", "b@example.com"), None);
    }

    #[test]
    fn a_malformed_username_shows_its_format_error_before_unchanged() {
        assert_eq!(change_username_error("a", "a"), username_error("a"));
    }

    #[test]
    fn a_tap_while_logging_in_sends_nothing() {
        assert_eq!(login_gate(true, "alice", "pw"), LoginGate::Busy);
        assert_eq!(login_gate(true, "", ""), LoginGate::Busy);
    }

    #[test]
    fn an_empty_or_blank_identifier_or_an_empty_password_asks_for_both() {
        for (identifier, password) in [
            ("", "pw"),
            ("   ", "pw"),
            ("\t\n", "pw"),
            ("alice", ""),
            ("", ""),
        ] {
            assert_eq!(
                login_gate(false, identifier, password),
                LoginGate::MissingInputs,
                "identifier {identifier:?}, password {password:?}"
            );
        }
    }

    #[test]
    fn login_applies_no_policy_and_a_password_of_spaces_still_sends() {
        assert_eq!(login_gate(false, "alice", "  "), LoginGate::Send);
        assert_eq!(login_gate(false, " alice ", "x"), LoginGate::Send);
    }

    #[test]
    fn the_missing_inputs_toast_is_an_error_at_the_normal_band() {
        assert_eq!(
            missing_inputs_toast(),
            Toast::new(ToastKind::Error, MISSING_INPUTS, crate::toast::NORMAL)
        );
    }

    #[test]
    fn the_reset_form_leaves_an_address_to_the_server() {
        for raw in ["a@b.co", "nope", " a@b.co", "a@localhost", "me@[127.0.0.1]"] {
            assert_eq!(reset_email_error(raw), None, "address {raw:?}");
        }
    }

    #[test]
    fn no_password_policy_is_checked_in_the_client() {
        assert_eq!(password_error("short", "short", true), None);
        assert_eq!(password_error("a", "a", false), None);
        assert_eq!(
            password_error("short", "other", true).as_deref(),
            Some(PASSWORDS_DIFFER)
        );
        assert_eq!(password_error("", "", false), None, "nothing tried yet");
        assert_eq!(
            password_error("", "", true).as_deref(),
            Some(PASSWORD_REQUIRED),
            "an empty submit asks for one"
        );
        assert_eq!(change_password_error(GOOD, "short", "short", true), None);
    }

    #[test]
    fn a_short_confirmation_waits_until_a_submit() {
        assert_eq!(password_error(GOOD, "", false), None);
        assert_eq!(password_error(GOOD, "Abc", false), None);
        assert_eq!(
            password_error(GOOD, "", true).as_deref(),
            Some(PASSWORDS_DIFFER)
        );
        assert_eq!(
            password_error(GOOD, "Abc", true).as_deref(),
            Some(PASSWORDS_DIFFER)
        );
        assert_eq!(
            password_error(GOOD, "Abcdef1?", false).as_deref(),
            Some(PASSWORDS_DIFFER),
            "as long as the password, so judged"
        );
        assert_eq!(
            password_error(GOOD, "Abcdef1!x", false).as_deref(),
            Some(PASSWORDS_DIFFER),
            "longer than the password, so judged"
        );
        assert_eq!(password_error(GOOD, GOOD, false), None);
        assert_eq!(password_error(GOOD, GOOD, true), None);
    }

    #[test]
    fn the_confirmation_is_compared_exactly_and_counted_in_characters() {
        assert_eq!(
            password_error(GOOD, "Abcdef1! ", false).as_deref(),
            Some(PASSWORDS_DIFFER),
            "a trailing space is a different password"
        );
        let accented = "Aé1!bcde";
        assert_eq!(
            password_error(accented, "Aé1!bcd", false),
            None,
            "seven characters, though eight bytes, is still typing"
        );
    }

    #[test]
    fn change_password_repeats_registers_checks_then_refuses_the_same_password() {
        assert_eq!(
            change_password_error(GOOD, GOOD, "Abc", true).as_deref(),
            Some(PASSWORDS_DIFFER),
            "the mismatch comes before sameness"
        );
        assert_eq!(
            change_password_error(GOOD, GOOD, GOOD, false).as_deref(),
            Some(PASSWORD_UNCHANGED)
        );
        assert_eq!(
            change_password_error(GOOD, GOOD, "Abc", false).as_deref(),
            Some(PASSWORD_UNCHANGED),
            "a confirmation still being typed leaves sameness to judge"
        );
    }

    #[test]
    fn resend_says_what_it_is_doing() {
        assert_eq!(resend_label(true, 60), "Sending…");
        assert_eq!(
            resend_label(true, 0),
            "Sending…",
            "sending wins over a spent cooldown"
        );
        assert_eq!(resend_label(false, 60), "Resend in 60s");
        assert_eq!(resend_label(false, 42), "Resend in 42s");
        assert_eq!(resend_label(false, 1), "Resend in 1s", "no singular form");
        assert_eq!(resend_label(false, 0), "Resend");
    }

    #[test]
    fn a_password_never_shows_in_debug() {
        let password = Password::new("hunter22");
        assert_eq!(format!("{password:?}"), "Password(REDACTED)");
        assert_eq!(password.read(), "hunter22");
        assert!(!password.is_empty());
        assert!(Password::default().is_empty());
    }

    #[test]
    fn delete_counts_down_then_unlocks() {
        assert_eq!(delete_label(5), "Delete (5)");
        assert_eq!(delete_label(2), "Delete (2)");
        assert_eq!(delete_label(1), "Delete (1)");
        assert_eq!(delete_label(0), "Delete");
    }
}
