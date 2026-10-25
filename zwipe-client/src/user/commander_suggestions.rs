//! Suggested commanders and their dismissals.

use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::{HttpCommanderSuggestions, HttpDismissCommander};
use zwipe_core::http::endpoints::suggestions::{
    ClearCommanderDismissals, DismissCommander, GetCommanderSuggestions, UndismissCommander,
};

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Reads the ranked suggestions, at most `limit` of them.
    pub async fn commander_suggestions(
        &self,
        limit: u32,
        session: &Session,
    ) -> Result<HttpCommanderSuggestions, ClientError> {
        self.call(GetCommanderSuggestions(limit), Some(session))
            .await
    }

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

    /// Lifts one dismissal.
    pub async fn undismiss_commander(
        &self,
        oracle_id: Uuid,
        session: &Session,
    ) -> Result<(), ClientError> {
        self.call(UndismissCommander(oracle_id), Some(session))
            .await
    }

    /// Lifts every dismissal; answers how many were removed.
    pub async fn clear_commander_dismissals(&self, session: &Session) -> Result<u32, ClientError> {
        self.call(ClearCommanderDismissals, Some(session))
            .await
            .map(|cleared| cleared.cleared)
    }
}
