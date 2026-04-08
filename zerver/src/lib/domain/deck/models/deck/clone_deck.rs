//! Clone deck operation.
//!
//! Service-layer error type for copying an existing deck (profile + all
//! entries on every board) into a new deck with a caller-chosen name.

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
#[cfg(feature = "zerver")]
use crate::outbound::sqlx::postgres::IsConstraintViolation;
#[cfg(feature = "zerver")]
use thiserror::Error;

/// Errors that can occur during deck cloning execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CloneDeckError {
    /// Source deck does not exist.
    #[error("source deck not found")]
    SourceNotFound,
    /// Caller does not own the source deck.
    #[error("cannot clone another user's deck")]
    Forbidden,
    /// A deck owned by the caller already has this name.
    #[error("a deck with that name already exists")]
    Duplicate,
    /// Caller has reached their deck-count limit.
    #[error("deck count limit reached")]
    LimitReached,
    /// Error surfaced while verifying the source deck exists and is owned.
    #[error(transparent)]
    GetSource(GetDeckProfileError),
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

/// Direct conversion from sqlx errors so the repo can use `?`.
///
/// Mirrors the pattern used by `CreateDeckProfileError` in
/// `outbound/sqlx/deck/error.rs`: a unique-constraint violation on
/// `unique_deck_name_per_user` maps to [`CloneDeckError::Duplicate`];
/// anything else is an opaque database error.
#[cfg(feature = "zerver")]
impl From<sqlx::Error> for CloneDeckError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            e if e.is_unique_constraint_violation() => Self::Duplicate,
            e => Self::Database(e.into()),
        }
    }
}
