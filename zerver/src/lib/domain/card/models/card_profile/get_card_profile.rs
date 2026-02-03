#[cfg(feature = "zerver")]
use crate::domain::card::models::scryfall_data::ScryfallData;
#[cfg(feature = "zerver")]
use thiserror::Error;
#[cfg(feature = "zerver")]
use uuid::Uuid;

// ========
//  errors
// ========

#[cfg(feature = "zerver")]
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

#[cfg(feature = "zerver")]
/// Errors that can occur when parsing card profile IDs.
#[derive(Debug, Error)]
pub enum InvalidCardProfileIds {
    /// UUID parsing failed.
    #[error("invalid id: {0}")]
    Uuid(uuid::Error),
    /// No IDs were provided (empty collection).
    #[error("no ids provided")]
    MissingIds,
}

#[cfg(feature = "zerver")]
impl From<uuid::Error> for InvalidCardProfileIds {
    fn from(value: uuid::Error) -> Self {
        Self::Uuid(value)
    }
}

// ==========
//  requests
// ==========

#[cfg(feature = "zerver")]
/// Request to get a single card profile by Scryfall ID.
///
/// Wraps a UUID parsed from a string ID.
pub struct GetCardProfile(Uuid);

#[cfg(feature = "zerver")]
impl GetCardProfile {
    /// Creates a new GetCardProfile request by parsing a UUID string.
    ///
    /// # Errors
    /// Returns `uuid::Error` if the string is not a valid UUID.
    pub fn new(id: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::try_parse(id)?))
    }

    /// Returns the parsed UUID.
    pub fn id(&self) -> Uuid {
        self.0
    }
}

#[cfg(feature = "zerver")]
/// Collection of card profile UUIDs for batch operations.
///
/// Used for bulk fetching card profiles by Scryfall IDs.
/// Derefs to `&[Uuid]` for direct slice operations.
#[derive(Debug)]
pub struct CardProfileIds(Vec<Uuid>);

#[cfg(feature = "zerver")]
impl std::ops::Deref for CardProfileIds {
    type Target = [Uuid];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "zerver")]
impl From<&[ScryfallData]> for CardProfileIds {
    fn from(value: &[ScryfallData]) -> Self {
        Self(value.iter().map(|sfd| sfd.id.to_owned()).collect())
    }
}
