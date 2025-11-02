use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetCardTypesError {
    #[error(transparent)]
    Database(anyhow::Error),
}
