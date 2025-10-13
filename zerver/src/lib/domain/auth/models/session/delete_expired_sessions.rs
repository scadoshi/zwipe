use crate::domain::auth::models::access_token::AccessToken;
use crate::domain::auth::models::refresh_token::RefreshToken;
use crate::domain::user::models::User;
#[cfg(feature = "zerver")]
use crate::domain::{auth::models::access_token::InvalidJwt, user::models::get_user::GetUserError};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DeleteExpiredSessionsError {
    #[error(transparent)]
    Database(anyhow::Error),
}
