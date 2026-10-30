//! Fetch the served keyword-reminder catalog (name → reminder text).

use std::collections::HashMap;

use zwipe_core::http::endpoints::card::GetKeywordReminders;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the keyword-reminder map. Served so definition fixes
    /// land on deploy instead of waiting for an app-store train; the compiled-in
    /// table stays as the offline fallback.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn get_keyword_reminders(&self) -> Result<HashMap<String, String>, ClientError> {
        self.call(GetKeywordReminders, None).await
    }
}
