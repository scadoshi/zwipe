pub mod get_user;
pub mod username;

use crate::domain::user::models::username::Username;
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
}

impl User {
    pub fn new(id: Uuid, username: Username, email: EmailAddress) -> Self {
        Self {
            id,
            username,
            email,
        }
    }
}
