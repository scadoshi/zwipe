//! Fetch the served keyword-reminder catalog (name → reminder text).

use crate::outbound::client::{ClientError, ZwipeClient};
use std::{collections::HashMap, future::Future};
use zwipe_core::http::endpoints::card::GetKeywordReminders;

/// Trait for fetching the keyword-reminder map. Served so definition fixes
/// land on deploy instead of waiting for an app-store train; the compiled-in
/// table stays as the offline fallback.
#[allow(missing_docs)]
pub trait ClientGetKeywordReminders {
    fn get_keyword_reminders(
        &self,
    ) -> impl Future<Output = Result<HashMap<String, String>, ClientError>> + Send;
}

impl ClientGetKeywordReminders for ZwipeClient {
    async fn get_keyword_reminders(&self) -> Result<HashMap<String, String>, ClientError> {
        self.call(GetKeywordReminders, None).await
    }
}
