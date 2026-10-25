//! Dismiss, undo and clear commander dismissals.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::HttpDismissCommander;
use zwipe_core::http::endpoints::suggestions::{
    ClearCommanderDismissals, DismissCommander, UndismissCommander,
};

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Hides a commander from the suggestions for a week.
    pub async fn dismiss_commander(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(
            DismissCommander(HttpDismissCommander { oracle_id }),
            Some(session),
        )
        .await
    }

    /// Undoes one dismissal.
    pub async fn undismiss_commander(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(UndismissCommander(oracle_id), Some(session))
            .await
    }

    /// Lifts every dismissal and answers how many were lifted.
    pub async fn clear_commander_dismissals(&self, session: &Session) -> Result<u32, ClientError> {
        self.call(ClearCommanderDismissals, Some(session))
            .await
            .map(|cleared| cleared.cleared)
    }
}
