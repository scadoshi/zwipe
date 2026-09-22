//! Commander maybeboard endpoints (per-user "maybe this commander" list).

use crate::{ClientError, ZwipeClient};
use uuid::Uuid;
use zwipe_core::{
    domain::{auth::models::session::Session, card::Card},
    http::endpoints::user::{
        AddCommanderMaybeboardCard, ClearCommanderMaybeboard, GetCommanderMaybeboard,
        RemoveCommanderMaybeboardCard,
    },
};

impl ZwipeClient {
    /// Reads the user's commander maybeboard.
    pub async fn get_commander_maybeboard(
        &self,
        session: &Session,
    ) -> Result<Vec<Card>, ClientError> {
        self.call(GetCommanderMaybeboard, Some(session)).await
    }

    /// Adds a commander to the maybeboard.
    pub async fn add_commander_maybeboard_card(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(AddCommanderMaybeboardCard(oracle_id), Some(session))
            .await
    }

    /// Removes a commander from the maybeboard.
    pub async fn remove_commander_maybeboard_card(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(RemoveCommanderMaybeboardCard(oracle_id), Some(session))
            .await
    }

    /// Empties the maybeboard.
    pub async fn clear_commander_maybeboard(&self, session: &Session) -> Result<(), ClientError> {
        self.call(ClearCommanderMaybeboard, Some(session)).await
    }
}
