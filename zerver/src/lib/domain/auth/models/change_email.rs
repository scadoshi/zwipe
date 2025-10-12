use email_address::EmailAddress;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidChangeEmail {
    #[error(transparent)]
    Id(uuid::Error),
    #[error(transparent)]
    Email(email_address::Error),
}

impl From<uuid::Error> for InvalidChangeEmail {
    fn from(value: uuid::Error) -> Self {
        Self::Id(value)
    }
}

impl From<email_address::Error> for InvalidChangeEmail {
    fn from(value: email_address::Error) -> Self {
        Self::Email(value)
    }
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ChangeEmailError {
    #[error("user not found")]
    UserNotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("user updated but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}

#[derive(Debug)]
pub struct ChangeEmail {
    pub user_id: Uuid,
    pub email: EmailAddress,
}

impl ChangeEmail {
    pub fn new(user_id: Uuid, email: &str) -> Result<Self, InvalidChangeEmail> {
        let email = EmailAddress::from_str(email)?;
        Ok(Self { user_id, email })
    }
}
