use email_address::EmailAddress;
use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::{
    domain::user::models::{username::Username, User},
    outbound::sqlx::user::error::IntoUserError,
};

/// raw database user record
/// (unvalidated data from `PostgreSQL`)
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUser {
    pub id: String,
    pub username: String,
    pub email: String,
}

/// converts database user to validated domain user
impl TryFrom<DatabaseUser> for User {
    type Error = IntoUserError;

    fn try_from(value: DatabaseUser) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id)?;
        let username = Username::new(&value.username)?;
        let email =
            EmailAddress::parse_with_options(&value.email, email_address::Options::default())?;
        Ok(Self {
            id,
            username,
            email,
        })
    }
}
