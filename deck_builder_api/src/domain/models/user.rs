use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx_macros::FromRow;

/// User model for authentication and deck ownership
#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
