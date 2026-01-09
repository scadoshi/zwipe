use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetArtistsError {
    #[error(transparent)]
    Database(anyhow::Error),
}
