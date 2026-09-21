//! Commander maybeboard endpoints (per-user "maybe this commander" list).

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use uuid::Uuid;
use zwipe_core::{
    domain::{auth::models::session::Session, card::Card},
    http::endpoints::user::{
        AddCommanderMaybeboardCard, ClearCommanderMaybeboard, GetCommanderMaybeboard,
        RemoveCommanderMaybeboardCard,
    },
};

/// Trait for reading and mutating the user's commander maybeboard.
#[allow(missing_docs)]
pub trait ClientCommanderMaybeboard {
    fn get_commander_maybeboard(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ClientError>> + Send;

    fn add_commander_maybeboard_card(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;

    fn remove_commander_maybeboard_card(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;

    fn clear_commander_maybeboard(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}

impl ClientCommanderMaybeboard for ZwipeClient {
    async fn get_commander_maybeboard(&self, session: &Session) -> Result<Vec<Card>, ClientError> {
        self.call(GetCommanderMaybeboard, Some(session)).await
    }

    async fn add_commander_maybeboard_card(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(AddCommanderMaybeboardCard(oracle_id), Some(session))
            .await
    }

    async fn remove_commander_maybeboard_card(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(RemoveCommanderMaybeboardCard(oracle_id), Some(session))
            .await
    }

    async fn clear_commander_maybeboard(&self, session: &Session) -> Result<(), ClientError> {
        self.call(ClearCommanderMaybeboard, Some(session)).await
    }
}
