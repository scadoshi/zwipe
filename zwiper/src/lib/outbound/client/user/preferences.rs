//! User preferences API client operations.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::{auth::models::session::Session, user::preferences::UserPreferences},
    http::{
        contracts::user::HttpUpdatePreferences,
        endpoints::user::{GetPreferences, UpdatePreferences},
    },
};

/// Trait for fetching user display preferences.
#[allow(missing_docs)]
pub trait ClientGetPreferences {
    fn get_preferences(
        &self,
        session: &Session,
    ) -> impl Future<Output = Result<UserPreferences, ClientError>> + Send;
}

/// Trait for updating user display preferences.
#[allow(missing_docs)]
pub trait ClientUpdatePreferences {
    fn update_preferences(
        &self,
        request: HttpUpdatePreferences,
        session: &Session,
    ) -> impl Future<Output = Result<UserPreferences, ClientError>> + Send;
}

impl ClientGetPreferences for ZwipeClient {
    async fn get_preferences(&self, session: &Session) -> Result<UserPreferences, ClientError> {
        self.call(GetPreferences, Some(session)).await
    }
}

impl ClientUpdatePreferences for ZwipeClient {
    async fn update_preferences(
        &self,
        request: HttpUpdatePreferences,
        session: &Session,
    ) -> Result<UserPreferences, ClientError> {
        let body = serde_json::to_value(&request)?;
        self.call(UpdatePreferences(body), Some(session)).await
    }
}
