#[cfg(feature = "zerver")]
use thiserror::Error;
use uuid::Uuid;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum DeleteUserError {
    #[error("user not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
}

#[derive(Debug, Clone)]
pub struct DeleteUser(Uuid);

impl DeleteUser {
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        let trimmed = id.trim();
        let id = Uuid::try_parse(trimmed)?;
        Ok(Self(id))
    }

    pub fn id(&self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for DeleteUser {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}
