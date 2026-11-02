//! The note under a pending email change, in the app's words. The rest of
//! Account's rules come from [`zwipe_ui::account`].

use chrono::{DateTime, Utc};

const SECS_PER_MINUTE: i64 = 60;
const MINUTES_PER_HOUR: i64 = 60;

/// The note under a pending email: minutes in the link's last hour, whole
/// hours (rounded) before that, and where to get a new link once it is dead.
/// Under a minute left reads "1 minute", as in the app; the link is dead once
/// no whole second is left. Computed when drawn; it does not tick.
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
        assert_eq!(at(Duration::seconds(59)), "Link expires in 1 minute");
        assert_eq!(at(Duration::seconds(1)), "Link expires in 1 minute");
    }

    #[test]
    fn a_dead_link_points_at_change() {
        let expired = "Link expired; tap Change to send a new one";
        assert_eq!(at(Duration::zero()), expired);
        assert_eq!(at(Duration::hours(-3)), expired);
    }
}
