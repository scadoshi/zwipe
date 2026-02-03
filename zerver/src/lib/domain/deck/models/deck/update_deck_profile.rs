//! Update deck profile operation.
//!
//! Allows users to modify deck metadata (name, commander, copy limit).
//! Uses partial update semantics - only specified fields are updated.
//!
//! # Partial Updates
//!
//! Fields use `Option<Option<T>>` to distinguish:
//! - `None`: Don't update this field (keep existing value)
//! - `Some(None)`: Set field to NULL (remove commander, use default copy limit)
//! - `Some(Some(value))`: Update to new value
//!
//! # Authorization
//!
//! Only the deck owner can update the deck (enforced by service layer).

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use crate::domain::deck::models::deck::{
    copy_max::{CopyMax, InvalidCopyMax},
    deck_name::{DeckName, InvalidDeckname},
};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing an [`UpdateDeckProfile`] request.
#[derive(Debug, Error)]
pub enum InvalidUpdateDeckProfile {
    /// Deck name doesn't meet requirements.
    #[error(transparent)]
    DeckName(InvalidDeckname),
    /// Copy limit is invalid (must be 1 or 4).
    #[error(transparent)]
    CopyMax(InvalidCopyMax),
    /// No fields specified for update.
    #[error("must update at least one field")]
    NoUpdates,
}

impl From<InvalidDeckname> for InvalidUpdateDeckProfile {
    fn from(value: InvalidDeckname) -> Self {
        Self::DeckName(value)
    }
}

impl From<InvalidCopyMax> for InvalidUpdateDeckProfile {
    fn from(value: InvalidCopyMax) -> Self {
        Self::CopyMax(value)
    }
}

/// Errors that can occur during deck profile update execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum UpdateDeckProfileError {
    /// User already has another deck with this name.
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    /// Deck ID doesn't exist.
    #[error("deck not found")]
    NotFound,
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Database returned invalid data after update.
    #[error("deck updated but database returned invalid object: {0}")]
    DeckFromDb(anyhow::Error),
    /// Error retrieving deck for authorization check.
    #[error(transparent)]
    GetDeckProfileError(#[from] GetDeckProfileError),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

/// Request to update deck profile metadata.
///
/// Uses partial update semantics with `Option<Option<T>>` for nullable fields:
/// - `None`: Don't change this field
/// - `Some(None)`: Set to NULL
/// - `Some(Some(value))`: Update to new value
///
/// # Example
///
/// ```rust,ignore
/// // Change name only
/// let update = UpdateDeckProfile::new(
///     deck_id,
///     Some("New Deck Name"),
///     None,  // Don't change commander
///     None,  // Don't change copy limit
///     user_id
/// )?;
///
/// // Remove commander
/// let update = UpdateDeckProfile::new(
///     deck_id,
///     None,
///     Some(None),  // Set commander to NULL
///     None,
///     user_id
/// )?;
/// ```
#[derive(Debug, Clone)]
pub struct UpdateDeckProfile {
    /// ID of deck to update.
    pub deck_id: Uuid,
    /// Optional new name.
    pub name: Option<DeckName>,
    /// Optional commander update (None = no change, Some(None) = remove commander, Some(Some(id)) = set commander).
    pub commander_id: Option<Option<Uuid>>,
    /// Optional copy limit update (None = no change, Some(None) = default, Some(Some(limit)) = set limit).
    pub copy_max: Option<Option<CopyMax>>,
    /// Requesting user (for authorization).
    pub user_id: Uuid,
}

impl UpdateDeckProfile {
    /// Creates a new deck profile update request with validation.
    ///
    /// # Parameters
    ///
    /// - `deck_id`: ID of deck to update
    /// - `name`: Optional new name
    /// - `commander_id`: Optional commander update
    /// - `copy_max`: Optional copy limit update
    /// - `user_id`: Requesting user ID
    ///
    /// # Errors
    ///
    /// Returns [`InvalidUpdateDeckProfile`] if:
    /// - Name doesn't meet requirements
    /// - Copy limit is not 1 or 4
    /// - No fields specified for update
    pub fn new(
        deck_id: Uuid,
        name: Option<&str>,
        commander_id: Option<Option<Uuid>>,
        copy_max: Option<Option<i32>>,
        user_id: Uuid,
    ) -> Result<Self, InvalidUpdateDeckProfile> {
        if name.is_none() && commander_id.is_none() && copy_max.is_none() {
            return Err(InvalidUpdateDeckProfile::NoUpdates);
        }
        let name = name.map(DeckName::new).transpose()?;
        let copy_max = copy_max
            .map(|update| update.map(CopyMax::new).transpose())
            .transpose()?;

        Ok(Self {
            deck_id,
            name,
            commander_id,
            copy_max,
            user_id,
        })
    }
}
