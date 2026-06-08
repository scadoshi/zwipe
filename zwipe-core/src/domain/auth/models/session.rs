//! Session entity shared between frontend and backend.
//!
//! A session represents a successful authentication with dual tokens:
//! - **Access Token**: Short-lived JWT (24h) for authenticating API requests
//! - **Refresh Token**: Long-lived token (14d) for obtaining new access tokens

use crate::domain::auth::models::access_token::AccessToken;
use crate::domain::auth::models::refresh_token::RefreshToken;
use crate::domain::user::models::User;
use crate::domain::user::models::preferences::UserPreferences;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Maximum number of concurrent sessions allowed per user.
///
/// When this limit is exceeded, the oldest session is automatically revoked.
pub const MAXIMUM_SESSION_COUNT: u8 = 5;

/// A successful authentication response containing user data and tokens.
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct Session {
    /// The authenticated user's public profile information.
    pub user: User,

    /// JWT access token for authenticating API requests (24h expiry).
    pub access_token: AccessToken,

    /// Long-lived refresh token for obtaining new access tokens (14d expiry).
    pub refresh_token: RefreshToken,

    /// User's display preferences (theme, dark mode).
    pub preferences: UserPreferences,
}

impl Session {
    /// Checks if the refresh token has expired.
    pub fn is_expired(&self) -> bool {
        self.refresh_token.expires_at < Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        auth::models::access_token::Jwt,
        user::models::{email::Email, username::Username},
    };
    use chrono::Duration;
    use std::str::FromStr;
    use uuid::Uuid;

    fn make_session(refresh_expires_at: chrono::DateTime<Utc>) -> Session {
        let user = User::new(
            Uuid::new_v4(),
            Username::new("alice").unwrap(),
            Email::from_str("alice@example.com").unwrap(),
        );
        let access_token = AccessToken {
            value: Jwt::from_str("header.payload.signature").unwrap(),
            expires_at: Utc::now() + Duration::hours(24),
        };
        let refresh_token = RefreshToken {
            value: "x".repeat(64),
            expires_at: refresh_expires_at,
        };
        Session {
            user,
            access_token,
            refresh_token,
            preferences: UserPreferences::default(),
        }
    }

    #[test]
    fn test_session_is_expired_when_refresh_token_has_past_expiry() {
        let past = Utc::now() - Duration::seconds(1);
        let session = make_session(past);
        assert!(session.is_expired());
    }

    #[test]
    fn test_session_is_not_expired_when_refresh_token_is_in_future() {
        let future = Utc::now() + Duration::days(14);
        let session = make_session(future);
        assert!(!session.is_expired());
    }
}
