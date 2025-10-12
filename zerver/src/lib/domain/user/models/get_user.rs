#[cfg(feature = "zerver")]
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetUserError {
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("user found but database returned invalid object: {0}")]
    UserFromDb(anyhow::Error),
}

#[derive(Debug, Clone)]
pub struct GetUser {
    pub user_id: Uuid,
}

impl GetUser {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self {
            user_id: Uuid::try_parse(id)?,
        })
    }
}

impl From<Uuid> for GetUser {
    fn from(value: Uuid) -> Self {
        Self { user_id: value }
    }
}
