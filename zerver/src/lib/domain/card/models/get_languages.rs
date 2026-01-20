use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetLanguagesError {
    #[error(transparent)]
    Database(anyhow::Error),
}
