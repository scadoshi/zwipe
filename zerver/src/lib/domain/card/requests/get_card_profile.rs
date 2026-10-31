use thiserror::Error;

// == errors ==

/// Errors that can occur when retrieving a card profile.
#[derive(Debug, Error)]
pub enum GetCardProfileError {
    /// Card profile was not found in database.
    #[error("card profile not found")]
    NotFound,
    /// Database query/connection error.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Card profile found but failed to deserialize from database row.
    #[error("card profile found but database returned invalid object: {0}")]
    CardProfileFromDb(anyhow::Error),
}
