//! Fetch the oracle tag catalog.

use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::http::endpoints::card::GetOracleTags;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the full oracle tag catalog (slug, label, description, parents).
    pub async fn get_oracle_tags(&self) -> Result<Vec<OracleTag>, ClientError> {
        self.call(GetOracleTags, None).await
    }
}
