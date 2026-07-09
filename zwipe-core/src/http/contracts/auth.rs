//! Authentication HTTP request contracts.

use serde::{Deserialize, Serialize};

use crate::domain::auth::models::platform::ClientPlatform;

/// Login request body.
///
/// `identifier` accepts either an email address or a username. `platform` is the
/// client's platform, recorded on the session; additive and `#[serde(default)]`
/// so older clients that omit it deserialize to `None`.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpAuthenticateUser {
    pub identifier: String,
    pub password: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<ClientPlatform>,
}

impl HttpAuthenticateUser {
    /// Creates a new login request (no platform; set `.platform` to record one).
    pub fn new(identifier: &str, password: &str) -> Self {
        Self {
            identifier: identifier.to_string(),
            password: password.to_string(),
            platform: None,
        }
    }
}

/// Registration request body. `platform` (additive, `#[serde(default)]`) is
/// recorded on the auto-login session created by registration.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRegisterUser {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<ClientPlatform>,
}

impl HttpRegisterUser {
    /// Creates a new registration request (no platform; set `.platform` to record one).
    pub fn new(username: &str, email: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            platform: None,
        }
    }
}

/// Token refresh request body.
///
/// `user_id` is a String that gets parsed to a Uuid.
/// On success the old refresh token is consumed and a new token pair is issued.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpRefreshSession {
    pub user_id: String,
    pub refresh_token: String,
}

impl HttpRefreshSession {
    /// Creates a new token refresh request.
    pub fn new(user_id: &str, refresh_token: &str) -> Self {
        Self {
            user_id: user_id.to_string(),
            refresh_token: refresh_token.to_string(),
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
