pub mod create_session;
pub mod delete_expired_sessions;
pub mod enforce_session_maximum;
pub mod refresh_session;
pub mod revoke_sessions;

use crate::domain::auth::models::access_token::AccessToken;
use crate::domain::auth::models::refresh_token::RefreshToken;
use crate::domain::user::models::User;
use chrono::Utc;
use serde::{Deserialize, Serialize};

pub const MAXIMUM_SESSION_COUNT: u8 = 5;

/// successful authentication response containing user data and tokens
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct Session {
    pub user: User,
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
}

impl Session {
    #[cfg(feature = "zerver")]
    pub fn new(user: User, access_token: AccessToken, refresh_token: RefreshToken) -> Self {
        Session {
            user,
            access_token,
            refresh_token,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.refresh_token.expires_at < Utc::now().naive_local()
    }
}
