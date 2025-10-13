use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnforceSessionMaximumError {
    #[error(transparent)]
    Database(anyhow::Error),
}
