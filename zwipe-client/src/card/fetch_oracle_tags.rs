//! Fetch the oracle tag catalog.

use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::http::endpoints::card::GetOracleTags;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the full oracle tag catalog (slug, label, description, parents).
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn fetch_oracle_tags(&self) -> Result<Vec<OracleTag>, ClientError> {
        self.call(GetOracleTags, None).await
    }
}
