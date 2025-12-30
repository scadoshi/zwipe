use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetSetsError {
    #[error(transparent)]
    Database(anyhow::Error),
}
