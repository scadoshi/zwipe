//! Account's words that depend on time: the pending email link's expiry,
//! the resend cooldown and the delete countdown.

use chrono::{DateTime, Utc};

/// Seconds before another verification email may go out. Matches the
/// server's resend limit (one, then one a minute per user), so the visible
/// countdown and a 429 agree.
pub const RESEND_COOLDOWN_SECS: u32 = 60;

/// Seconds the Delete button stays disabled after the dialog opens.
pub const DELETE_COUNTDOWN_SECS: u8 = 5;

const SECS_PER_MINUTE: i64 = 60;
const MINUTES_PER_HOUR: i64 = 60;

/// The note under a pending email: minutes in the link's last hour, whole
/// hours (rounded) before that, and where to get a new link once it is dead.
/// Computed when drawn; it does not tick.
pub fn link_expiry_text(expires_at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let left = expires_at.signed_duration_since(now).num_seconds();
    if left <= 0 {
        return "Link expired; tap Change to send a new one".to_string();
    }
    let minutes = left.saturating_add(SECS_PER_MINUTE - 1) / SECS_PER_MINUTE;
    if minutes < MINUTES_PER_HOUR {
        return match minutes {
            1 => "Link expires in 1 minute".to_string(),
            n => format!("Link expires in {n} minutes"),
        };
    }
    match minutes.saturating_add(MINUTES_PER_HOUR / 2) / MINUTES_PER_HOUR {
        1 => "Link expires in 1 hour".to_string(),
        n => format!("Link expires in {n} hours"),
    }
}

/// The Resend button's label while a send is out, while the cooldown runs,
/// and otherwise.
pub fn resend_label(sending: bool, cooldown: u32) -> String {
    if sending {
        "Sending...".to_string()
    } else if cooldown > 0 {
        format!("Resend in {cooldown}s")
    } else {
        "Resend".to_string()
    }
}

/// The Delete button's label with `remaining` seconds on its countdown.
pub fn delete_label(remaining: u8) -> String {
    match remaining {
        0 => "Delete".to_string(),
        n => format!("Delete ({n})"),
    }
}

#[cfg(test)]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "test fixtures; overflow fails the test"
)]
mod tests {
    use chrono::{Duration, TimeZone};

    use super::*;

    fn at(left: Duration) -> String {
        let now = Utc.with_ymd_and_hms(2026, 10, 23, 12, 0, 0).unwrap();
        link_expiry_text(now + left, now)
    }

    #[test]
    fn a_fresh_link_reads_in_hours() {
        assert_eq!(at(Duration::hours(24)), "Link expires in 24 hours");
        assert_eq!(
            at(Duration::hours(24) - Duration::seconds(5)),
            "Link expires in 24 hours"
        );
        assert_eq!(at(Duration::minutes(90)), "Link expires in 2 hours");
        assert_eq!(at(Duration::minutes(80)), "Link expires in 1 hour");
        assert_eq!(at(Duration::minutes(60)), "Link expires in 1 hour");
        assert_eq!(
            at(Duration::seconds(3599)),
            "Link expires in 1 hour",
            "60 minutes once rounded up"
        );
    }

    #[test]
    fn the_last_hour_reads_in_minutes() {
        assert_eq!(at(Duration::minutes(59)), "Link expires in 59 minutes");
        assert_eq!(at(Duration::seconds(61)), "Link expires in 2 minutes");
        assert_eq!(at(Duration::seconds(1)), "Link expires in 1 minute");
    }

    #[test]
    fn a_dead_link_points_at_change() {
        let expired = "Link expired; tap Change to send a new one";
        assert_eq!(at(Duration::zero()), expired);
        assert_eq!(at(Duration::hours(-3)), expired);
    }

    #[test]
    fn resend_says_what_it_is_doing() {
        assert_eq!(resend_label(true, 60), "Sending...");
        assert_eq!(resend_label(false, 42), "Resend in 42s");
        assert_eq!(resend_label(false, 0), "Resend");
    }

    #[test]
    fn delete_counts_down_then_unlocks() {
        assert_eq!(delete_label(5), "Delete (5)");
        assert_eq!(delete_label(1), "Delete (1)");
        assert_eq!(delete_label(0), "Delete");
    }
}
