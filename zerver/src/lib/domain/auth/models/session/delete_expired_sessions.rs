use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeleteExpiredSessionsError {
    #[error(transparent)]
    Database(anyhow::Error),
}
