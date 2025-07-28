use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::users;

/// User model for authentication and deck ownership
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,

    // Don't serialize password hash for security
    #[serde(skip_serializing)]
    pub password_hash: String,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// New user for registration (before saving to database)
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUser {
    pub email: String,
    pub username: String,
    pub password_hash: String,
}

/// User data for updates (all fields optional)
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateUser {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password_hash: Option<String>,
}

/// Public user info for API responses (no sensitive data)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub created_at: NaiveDateTime,
}
