//! When a session can back a call as it is, and when it must be refreshed or
//! dropped.

use chrono::Duration;
use zwipe_client::ClientError;
use zwipe_core::domain::auth::models::session::Session;

/// How long before `expires_at` the access token counts as stale. Covers the
/// server's 60-second leeway and a clock up to a minute slow.
pub const REFRESH_AHEAD: Duration = Duration::minutes(2);

/// The state of the stored session before an authed call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    /// No session: the call cannot go out.
    Missing,
    /// The refresh token is past its expiry: drop the session.
    Expired,
    /// The access token is good for at least [`REFRESH_AHEAD`].
    Fresh,
    /// The access token expires within [`REFRESH_AHEAD`]: refresh first.
    Stale,
}

/// Classifies `session`, checking the refresh token before the access token.
pub fn freshness(session: Option<&Session>) -> Freshness {
    match session {
        None => Freshness::Missing,
        Some(session) if session.is_expired() => Freshness::Expired,
        Some(session) if session.access_token.expires_within(REFRESH_AHEAD) => Freshness::Stale,
        Some(_) => Freshness::Fresh,
    }
}

/// Whether a failed refresh means the session is gone for good. Any other
/// failure keeps it, and the next call tries again.
pub fn refresh_rejected(error: &ClientError) -> bool {
    matches!(
        error,
        ClientError::Unauthorized(_) | ClientError::Forbidden(_) | ClientError::NotFound(_)
    )
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::test_fixtures;

    #[test]
    fn no_session_is_missing() {
        assert_eq!(freshness(None), Freshness::Missing);
    }

    #[test]
    fn a_new_session_is_fresh() {
        assert_eq!(freshness(Some(&test_fixtures::session())), Freshness::Fresh);
    }

    #[test]
    fn an_access_token_inside_the_window_is_stale() {
        let mut session = test_fixtures::session();
        session.access_token.expires_at = Utc::now() + Duration::seconds(90);
        assert_eq!(freshness(Some(&session)), Freshness::Stale);
    }

    #[test]
    fn a_dead_refresh_token_wins_over_a_stale_access_token() {
        let mut session = test_fixtures::session();
        session.access_token.expires_at = Utc::now() - Duration::minutes(1);
        session.refresh_token.expires_at = Utc::now() - Duration::minutes(1);
        assert_eq!(freshness(Some(&session)), Freshness::Expired);
    }

    #[test]
    fn only_auth_rejections_drop_the_session() {
        let says = || "no".to_string();
        assert!(refresh_rejected(&ClientError::Unauthorized(says())));
        assert!(refresh_rejected(&ClientError::Forbidden(says())));
        assert!(refresh_rejected(&ClientError::NotFound(says())));
        assert!(!refresh_rejected(&ClientError::InternalServerError(says())));
        assert!(!refresh_rejected(&ClientError::TooManyRequests(says())));
    }
}
