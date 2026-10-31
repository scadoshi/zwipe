//! Domain values for unit tests. zwipe-core's own builders sit behind its
//! `cfg(test)`, so the tests build a session here.

#![expect(
    clippy::arithmetic_side_effects,
    reason = "test fixtures; overflow fails the test"
)]

use std::str::FromStr;

use chrono::{Duration, Utc};
use uuid::Uuid;
use zwipe_core::domain::auth::models::access_token::{AccessToken, Jwt};
use zwipe_core::domain::auth::models::refresh_token::RefreshToken;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::User;
use zwipe_core::domain::user::models::email::Email;
use zwipe_core::domain::user::models::preferences::UserPreferences;
use zwipe_core::domain::user::models::username::Username;

/// A live session for a made-up user; the tokens are well formed but sign
/// nothing.
pub(crate) fn session() -> Session {
    Session {
        user: User::new(
            Uuid::new_v4(),
            Username::new("alice").expect("a valid username"),
            Email::from_str("alice@example.com").expect("a valid email"),
        ),
        access_token: AccessToken {
            value: Jwt::from_str("header.payload.signature").expect("a well-formed jwt"),
            expires_at: Utc::now() + Duration::hours(24),
        },
        refresh_token: RefreshToken {
            value: "x".repeat(64),
            expires_at: Utc::now() + Duration::days(14),
        },
        preferences: UserPreferences::default(),
    }
}
