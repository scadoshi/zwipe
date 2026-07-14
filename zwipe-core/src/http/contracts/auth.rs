//! Authentication HTTP request contracts.

use serde::{Deserialize, Serialize};

use crate::domain::auth::models::platform::ClientPlatform;

/// Login request body.
///
/// `identifier` accepts either an email address or a username. `platform` is the
/// client's platform and `client_version` its app version (e.g. `"1.6.1"`), both
/// recorded on the session; additive and `#[serde(default)]` so older clients
/// that omit them deserialize to `None`.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpAuthenticateUser {
    pub identifier: String,
    pub password: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<ClientPlatform>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_version: Option<String>,
}

impl HttpAuthenticateUser {
    /// Creates a new login request (no platform/version; set the fields to record them).
    pub fn new(identifier: &str, password: &str) -> Self {
        Self {
            identifier: identifier.to_string(),
            password: password.to_string(),
            platform: None,
            client_version: None,
        }
    }
}

/// Registration request body. `platform` and `client_version` (additive,
/// `#[serde(default)]`) are recorded on the auto-login session created by
/// registration.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRegisterUser {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<ClientPlatform>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_version: Option<String>,
}

impl HttpRegisterUser {
    /// Creates a new registration request (no platform/version; set the fields to record them).
    pub fn new(username: &str, email: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            platform: None,
            client_version: None,
        }
    }
}

/// Token refresh request body.
///
/// `user_id` is a String that gets parsed to a Uuid. `client_version` (additive,
/// `#[serde(default)]`) is the app version currently running; the client re-sends
/// it on every refresh so the rotated session reflects the live version rather
/// than the one it was first created with.
/// On success the old refresh token is consumed and a new token pair is issued.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRefreshSession {
    pub user_id: String,
    pub refresh_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_version: Option<String>,
}

impl HttpRefreshSession {
    /// Creates a new token refresh request (no version; set `.client_version` to record one).
    pub fn new(user_id: &str, refresh_token: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            refresh_token: refresh_token.to_string(),
            client_version: None,
        }
    }
}

/// Password change request body. Requires current password for re-verification.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpChangePassword {
    pub current_password: String,
    pub new_password: String,
}

impl HttpChangePassword {
    /// Creates a new password change request.
    pub fn new(current_password: &str, new_password: &str) -> Self {
        Self {
            current_password: current_password.to_string(),
            new_password: new_password.to_string(),
        }
    }
}

/// Username change request body. Requires password for re-verification.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpChangeUsername {
    pub new_username: String,
    pub password: String,
}

impl HttpChangeUsername {
    /// Creates a new username change request.
    pub fn new(new_username: &str, password: &str) -> Self {
        Self {
            new_username: new_username.to_string(),
            password: password.to_string(),
        }
    }
}

/// Email change request body. Requires password for re-verification.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpChangeEmail {
    pub email: String,
    pub password: String,
}

impl HttpChangeEmail {
    /// Creates a new email change request.
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

/// Account deletion request body. Requires password confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpDeleteUser {
    /// Current password to confirm deletion.
    pub password: String,
}

/// Email verification request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpVerifyEmail {
    pub token: String,
}

/// Password reset initiation request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRequestPasswordReset {
    pub email: String,
}

impl HttpRequestPasswordReset {
    /// Creates a new password reset request for the given email address.
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
        }
    }
}

/// Password reset completion request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpResetPassword {
    pub token: String,
    pub new_password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // The `platform` / `client_version` fields are additive: older clients omit
    // them and must still deserialize (to `None`), and we don't emit noise on the
    // wire when they're unset. These tests pin that backward-compatibility
    // contract so a future `#[serde(default)]` removal can't slip through.

    #[test]
    fn login_deserializes_without_client_version() {
        let json = r#"{"identifier":"alice","password":"pw"}"#;
        let req: HttpAuthenticateUser = serde_json::from_str(json).unwrap();
        assert_eq!(req.client_version, None);
        assert_eq!(req.platform, None);
    }

    #[test]
    fn login_roundtrips_client_version() {
        let mut req = HttpAuthenticateUser::new("alice", "pw");
        req.client_version = Some("1.6.1".to_string());
        let wire = serde_json::to_string(&req).unwrap();
        let back: HttpAuthenticateUser = serde_json::from_str(&wire).unwrap();
        assert_eq!(back.client_version.as_deref(), Some("1.6.1"));
    }

    #[test]
    fn login_omits_client_version_when_none() {
        let req = HttpAuthenticateUser::new("alice", "pw");
        let wire = serde_json::to_string(&req).unwrap();
        assert!(!wire.contains("client_version"), "wire: {wire}");
    }

    #[test]
    fn register_deserializes_without_client_version() {
        let json = r#"{"username":"alice","email":"a@b.co","password":"pw"}"#;
        let req: HttpRegisterUser = serde_json::from_str(json).unwrap();
        assert_eq!(req.client_version, None);
    }

    #[test]
    fn refresh_deserializes_without_client_version() {
        let json = r#"{"user_id":"u","refresh_token":"t"}"#;
        let req: HttpRefreshSession = serde_json::from_str(json).unwrap();
        assert_eq!(req.client_version, None);
    }

    #[test]
    fn refresh_roundtrips_client_version() {
        let mut req = HttpRefreshSession::new("u", "t");
        req.client_version = Some("1.6.2".to_string());
        let wire = serde_json::to_string(&req).unwrap();
        let back: HttpRefreshSession = serde_json::from_str(&wire).unwrap();
        assert_eq!(back.client_version.as_deref(), Some("1.6.2"));
    }
}
