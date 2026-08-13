//! Commander maybeboard entry operations (per-user "maybe this commander").

use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`CommanderMaybeboardCard`]
/// request.
#[derive(Debug, Error)]
pub enum InvalidCommanderMaybeboardCard {
    /// Invalid oracle ID format.
    #[error(transparent)]
    OracleId(uuid::Error),
}

/// Request to add or remove a single commander maybeboard entry for a user.
#[derive(Debug, Clone)]
pub struct CommanderMaybeboardCard {
    /// Owning user.
    pub user_id: Uuid,
    /// Oracle id of the commander (covers all printings).
    pub oracle_id: Uuid,
}

impl CommanderMaybeboardCard {
    /// Creates a new commander maybeboard entry request.
    pub fn new(user_id: Uuid, oracle_id: Uuid) -> Self {
        Self { user_id, oracle_id }
    }

    /// Creates a new entry request parsing the oracle id from a path segment.
    pub fn from_path(
        user_id: Uuid,
        oracle_id: &str,
    ) -> Result<Self, InvalidCommanderMaybeboardCard> {
        let oracle_id =
            Uuid::try_parse(oracle_id.trim()).map_err(InvalidCommanderMaybeboardCard::OracleId)?;
        Ok(Self::new(user_id, oracle_id))
    }
}
