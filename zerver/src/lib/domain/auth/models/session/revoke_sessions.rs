use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidRevokeSessions {
    #[error(transparent)]
    UserId(uuid::Error),
}

impl From<uuid::Error> for InvalidRevokeSessions {
    fn from(value: uuid::Error) -> Self {
        Self::UserId(value)
    }
}

#[derive(Debug, Error)]
pub enum RevokeSessionsError {
    #[error(transparent)]
    Database(anyhow::Error),
}

#[derive(Debug, Clone)]
pub struct RevokeSessions {
    pub user_id: Uuid,
}

impl RevokeSessions {
    pub fn new(user_id: Uuid) -> Self {
        Self { user_id }
    }
}

impl FromStr for RevokeSessions {
    type Err = InvalidRevokeSessions;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let user_id = Uuid::try_parse(s)?;
        Ok(Self { user_id })
    }
}

impl From<Uuid> for RevokeSessions {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}
