//! JWT access token types shared between frontend and backend.
//!
//! This module provides the pure data types for JWT-based authentication:
//! - [`Jwt`]: A validated JWT token string (format-checked)
//! - [`AccessToken`]: JWT value + expiry timestamp
//! - [`UserClaims`]: User data embedded in the JWT payload
//! - [`InvalidJwt`]: Validation errors for JWT format
//!
//! Server-only operations (signing, verification) are provided by zerver
//! via extension traits.

use crate::domain::user::models::{email::Email, username::Username};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

// ========
//  errors
// ========

/// Errors when parsing a JWT token string.
#[derive(Debug, Clone, Error)]
pub enum InvalidJwt {
    /// No token was provided (empty string).
    #[error("token must be present")]
    MissingToken,

    /// Token doesn't have the required JWT format (header.payload.signature).
    #[error("invalid token format")]
    Format,
}

// ==========
//  newtypes
// ==========

/// JWT claims containing user information.
///
/// These claims are embedded in the JWT payload and used to identify the
/// authenticated user without database lookups on every request.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct UserClaims {
    /// Unique identifier for the authenticated user.
    pub user_id: Uuid,

    /// Username of the authenticated user (cached from database).
    pub username: Username,

    /// Email of the authenticated user (cached from database).
    pub email: Email,

    /// Expiry time as Unix timestamp (seconds since epoch).
    pub exp: i64,

    /// Issued at time as Unix timestamp (seconds since epoch).
    pub iat: i64,
}

// ======
//  main
// ======

/// A validated JWT token string with the correct format.
///
/// Guarantees the string has `header.payload.signature` structure
/// (exactly 3 base64-encoded parts separated by dots).
///
/// Signature and expiry validation requires server-side operations
/// not available in this crate.
#[derive(Debug, Clone, PartialEq)]
pub struct Jwt(String);

impl FromStr for Jwt {
    type Err = InvalidJwt;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(InvalidJwt::MissingToken);
        }
        if s.split('.').count() != 3 {
            return Err(InvalidJwt::Format);
        }
        Ok(Self(s.to_string()))
    }
}

impl std::ops::Deref for Jwt {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Jwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Jwt {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self)
    }
}

impl<'de> Deserialize<'de> for Jwt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Jwt::from_str(s.as_str()).map_err(serde::de::Error::custom)
    }
}

/// A complete access token with JWT value and expiry time.
///
/// This is the primary authentication credential returned to clients after login
/// or registration. Contains both the JWT token string and metadata about when
/// it expires.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessToken {
    /// The JWT token string.
    pub value: Jwt,

    /// When the token expires (24 hours from issuance).
    pub expires_at: NaiveDateTime,
}

impl AccessToken {
    /// Checks if the access token has expired (current time >= expires_at).
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now().naive_utc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // ================
    //  `Jwt` tests
    // ================

    #[test]
    fn test_access_token_new_accepts_valid_token() {
        let token = Jwt::from_str("header.payload.signature");
        assert!(token.is_ok());
    }

    #[test]
    fn test_access_token_new_rejects_empty_token() {
        let token = Jwt::from_str("");
        assert!(token.is_err());
        assert!(matches!(token.unwrap_err(), InvalidJwt::MissingToken));
    }

    #[test]
    fn test_access_token_new_rejects_token_with_too_few_parts() {
        let token = Jwt::from_str("header.payload");
        assert!(token.is_err());
        assert!(matches!(token.unwrap_err(), InvalidJwt::Format));
    }

    #[test]
    fn test_access_token_new_rejects_token_with_too_many_parts() {
        let token = Jwt::from_str("header.payload.signature.extra");
        assert!(token.is_err());
        assert!(matches!(token.unwrap_err(), InvalidJwt::Format));
    }

    #[test]
    fn test_access_token_display_formats_correctly() {
        let token_str = "header.payload.signature";
        let token = Jwt::from_str(token_str).unwrap();
        assert_eq!(token.to_string(), token_str);
    }

    #[test]
    fn test_access_token_partial_eq_works() {
        let token1 = Jwt::from_str("header.payload.signature").unwrap();
        let token2 = Jwt::from_str("header.payload.signature").unwrap();
        let token3 = Jwt::from_str("different.payload.signature").unwrap();

        assert_eq!(token1, token2);
        assert_ne!(token1, token3);
    }

    // ====================
    //  `UserClaims` tests
    // ====================

    #[test]
    fn test_user_claims_serialization_round_trip() {
        let user_id = uuid::Uuid::new_v4();
        let username = Username::new("test").unwrap();
        let email = Email::from_str("test@example.com").unwrap();
        let claims = UserClaims {
            user_id,
            username,
            email,
            exp: 1234567890,
            iat: 1234567890,
        };

        let serialized = serde_json::to_string(&claims).unwrap();
        let deserialized: UserClaims = serde_json::from_str(&serialized).unwrap();

        assert_eq!(claims, deserialized);
    }

    #[test]
    fn test_user_claims_partial_eq_works() {
        let id1 = uuid::Uuid::new_v4();
        let id2 = uuid::Uuid::new_v4();
        let username = Username::new("test").unwrap();
        let email = Email::from_str("test@example.com").unwrap();
        let claims1 = UserClaims {
            user_id: id1,
            username: username.clone(),
            email: email.clone(),
            exp: 1234567890,
            iat: 1234567890,
        };
        let claims2 = UserClaims {
            user_id: id1,
            username: username.clone(),
            email: email.clone(),
            exp: 1234567890,
            iat: 1234567890,
        };
        let claims3 = UserClaims {
            user_id: id2,
            username,
            email,
            exp: 1234567890,
            iat: 1234567890,
        };

        assert_eq!(claims1, claims2);
        assert_ne!(claims1, claims3);
    }

    // ==============================
    //  `AccessToken::is_expired` tests
    // ==============================

    #[test]
    fn test_access_token_is_not_expired_when_expiry_is_in_future() {
        let token = AccessToken {
            value: Jwt::from_str("header.payload.signature").unwrap(),
            expires_at: Utc::now().naive_utc() + chrono::Duration::hours(24),
        };
        assert!(!token.is_expired());
    }

    #[test]
    fn test_access_token_is_expired_when_expiry_is_in_past() {
        let token = AccessToken {
            value: Jwt::from_str("header.payload.signature").unwrap(),
            expires_at: Utc::now().naive_utc() - chrono::Duration::seconds(1),
        };
        assert!(token.is_expired());
    }
}
