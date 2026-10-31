//! The session rules every client applies before an authed call, as the
//! app's `ensure_session` does, and the version gate its upkeep runs. The
//! client keeps the session, stores it and runs the refresh; these decide
//! when.

use chrono::{DateTime, Duration, Utc};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::version::version_at_least;

/// How long before it expires an access token counts as stale: the
/// server's 60-second leeway plus a device clock up to a minute slow.
pub const REFRESH_AHEAD: Duration = Duration::minutes(2);

/// Where a session stands before an authed call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Freshness {
    /// The refresh token is dead: clear the session and send the user to
    /// log in.
    Expired,
    /// The access token expires within [`REFRESH_AHEAD`]: refresh first.
    Stale,
    /// Good to use as is.
    Fresh,
}

/// Where `session` stands at `now`. The refresh token is checked first, so a
/// session whose refresh token is dead is never handed out, however long its
/// access token has left.
#[must_use]
pub fn freshness(session: &Session, now: DateTime<Utc>) -> Freshness {
    if session.refresh_token.expires_at < now {
        return Freshness::Expired;
    }
    let stale = now
        .checked_add_signed(REFRESH_AHEAD)
        .is_none_or(|limit| session.access_token.expires_at < limit);
    if stale {
        Freshness::Stale
    } else {
        Freshness::Fresh
    }
}

/// Whether a failed refresh, answered with HTTP `status`, ends the session.
/// Unauthorized, Forbidden and Not Found mean the server will never take
/// that refresh token again; anything else is transient and keeps the
/// session for the next try.
#[must_use]
pub fn refresh_rejected(status: u16) -> bool {
    matches!(status, 401 | 403 | 404)
}

/// Whether the update-required screen shows after a version check: the
/// minimum the server answered with, or `None` when the check failed, which
/// leaves the gate as it was.
#[must_use]
pub fn update_required(was_required: bool, version: &str, minimum: Option<&str>) -> bool {
    minimum.map_or(was_required, |minimum| !version_at_least(version, minimum))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(access_in: Duration, refresh_in: Duration, now: DateTime<Utc>) -> Session {
        serde_json::from_value(serde_json::json!({
            "user": {
                "id": "00000000-0000-0000-0000-000000000001",
                "username": "scotty",
                "email": "scotty@zwipe.net",
                "email_verified_at": null,
            },
            "access_token": {"value": "aGVhZA.cGF5bG9hZA.c2ln", "expires_at": now.checked_add_signed(access_in).unwrap()},
            "refresh_token": {"value": "r", "expires_at": now.checked_add_signed(refresh_in).unwrap()},
            "preferences": zwipe_core::domain::user::UserPreferences::default(),
        }))
        .unwrap()
    }

    #[test]
    fn a_dead_refresh_token_expires_the_session_whatever_the_access_token() {
        let now = Utc::now();
        assert_eq!(
            freshness(
                &session(Duration::hours(10), Duration::seconds(-1), now),
                now
            ),
            Freshness::Expired,
            "checked first, as in the app"
        );
    }

    #[test]
    fn an_access_token_near_its_end_needs_a_refresh() {
        let now = Utc::now();
        let days = Duration::days(10);
        assert_eq!(
            freshness(&session(Duration::minutes(1), days, now), now),
            Freshness::Stale
        );
        assert_eq!(
            freshness(&session(Duration::minutes(3), days, now), now),
            Freshness::Fresh
        );
    }

    #[test]
    fn only_auth_answers_end_a_session() {
        assert!(refresh_rejected(401) && refresh_rejected(403) && refresh_rejected(404));
        assert!(!refresh_rejected(500) && !refresh_rejected(429));
    }

    #[test]
    fn the_version_gate_opens_closes_and_holds_on_a_failed_check() {
        assert!(update_required(false, "1.15.0", Some("1.16.0")));
        assert!(!update_required(true, "1.16.0", Some("1.16.0")));
        assert!(
            update_required(true, "1.16.0", None),
            "fails open: no change"
        );
        assert!(!update_required(false, "1.16.0", None));
    }
}
