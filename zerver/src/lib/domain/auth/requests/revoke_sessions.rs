//! Session revocation: logging out one session, or every session a user holds.
//!
//! A session is one refresh token family, the chain of tokens that one login
//! starts and every rotation carries. [`RevokeScope::Family`] ends one of
//! them, which is what a logout does. [`RevokeScope::Everywhere`] ends them
//! all, which a logout falls back to when its request names no family.
//!
//! After revocation, a revoked session's access tokens still pass the
//! signature check until they expire, but its refresh tokens are gone, so it
//! cannot get new ones and must sign in again.
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::auth::requests::revoke_sessions::RevokeSessions;
//!
//! // End the session the access token belongs to.
//! session_service.revoke_sessions(&RevokeSessions::family(user_id, family_id)).await?;
//!
//! // End every session the user holds.
//! session_service.revoke_sessions(&RevokeSessions::new(user_id)).await?;
//! ```

use std::str::FromStr;

use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`RevokeSessions`] request.
#[derive(Debug, Error)]
pub enum InvalidRevokeSessions {
    /// Invalid user ID format.
    #[error(transparent)]
    UserId(uuid::Error),
}

impl From<uuid::Error> for InvalidRevokeSessions {
    fn from(value: uuid::Error) -> Self {
        Self::UserId(value)
    }
}

/// Errors that can occur during session revocation execution.
#[derive(Debug, Error)]
pub enum RevokeSessionsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

/// Which of the user's sessions a revocation ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevokeScope {
    /// Every session the user holds.
    Everywhere,
    /// The one session whose refresh tokens share this family id. A family
    /// the user does not hold ends nothing.
    Family(Uuid),
}

/// Request to revoke a user's sessions: one family, or all of them.
#[derive(Debug, Clone)]
pub struct RevokeSessions {
    /// The user whose sessions should be revoked.
    pub user_id: Uuid,
    /// Which of them.
    pub scope: RevokeScope,
}

impl RevokeSessions {
    /// Revokes every session `user_id` holds.
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            scope: RevokeScope::Everywhere,
        }
    }

    /// Revokes the session of `family_id`, or every session `user_id` holds
    /// when there is no family to name.
    pub fn family(user_id: Uuid, family_id: Option<Uuid>) -> Self {
        Self {
            user_id,
            scope: family_id.map_or(RevokeScope::Everywhere, RevokeScope::Family),
        }
    }
}

impl FromStr for RevokeSessions {
    type Err = InvalidRevokeSessions;

    /// Parses a user ID string into a revocation request.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidRevokeSessions::UserId`] if the string is not a valid UUID.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let user_id = Uuid::try_parse(s)?;
        Ok(Self::new(user_id))
    }
}

impl From<Uuid> for RevokeSessions {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use uuid::Uuid;

    use super::*;

    const VALID_UUID: &str = "550e8400-e29b-41d4-a716-446655440000";

    #[test]
    fn test_revoke_sessions_new_stores_user_id() {
        let user_id = Uuid::new_v4();
        let req = RevokeSessions::new(user_id);
        assert_eq!(req.user_id, user_id);
    }

    #[test]
    fn a_family_scopes_the_revocation_and_none_falls_back_to_everywhere() {
        let (user_id, family) = (Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(
            RevokeSessions::family(user_id, Some(family)).scope,
            RevokeScope::Family(family)
        );
        assert_eq!(
            RevokeSessions::family(user_id, None).scope,
            RevokeScope::Everywhere
        );
        assert_eq!(RevokeSessions::new(user_id).scope, RevokeScope::Everywhere);
    }

    #[test]
    fn test_revoke_sessions_from_uuid() {
        let user_id = Uuid::new_v4();
        let req = RevokeSessions::from(user_id);
        assert_eq!(req.user_id, user_id);
    }

    #[test]
    fn test_revoke_sessions_from_str_succeeds_with_valid_uuid() {
        let result = RevokeSessions::from_str(VALID_UUID);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().user_id.to_string(), VALID_UUID);
    }

    #[test]
    fn test_revoke_sessions_from_str_rejects_invalid_uuid() {
        let result = RevokeSessions::from_str("not-a-uuid");
        assert!(matches!(result, Err(InvalidRevokeSessions::UserId(_))));
    }
}
